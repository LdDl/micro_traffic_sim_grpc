use std::collections::HashMap;
use std::env;

use tokio_stream::StreamExt;
use tonic::transport::Channel;

use micro_traffic_sim::pb;
use micro_traffic_sim::pb::service_client::ServiceClient;
use micro_traffic_sim::record::decode_record_batch;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw = env::var("MT_SIM_ADDR").unwrap_or_else(|_| "127.0.0.1:50051".to_string());
    let addr = if raw.starts_with("http://") || raw.starts_with("https://") {
        raw
    } else {
        format!("http://{raw}")
    };
    let channel = Channel::from_shared(addr)?.connect().await?;
    let mut client = ServiceClient::new(channel);

    let resp = client
        .new_session(pb::SessionReq { srid: 0 })
        .await?
        .into_inner();
    let sid = resp
        .id
        .as_ref()
        .map(|x| x.value.clone())
        .ok_or("server returned empty session id")?;
    eprintln!("session: {sid}");

    const ZONE_BIRTH: i32 = 1;
    const ZONE_DEATH: i32 = 2;
    const ZONE_COMMON: i32 = 4;

    let mut cells: Vec<pb::Cell> = Vec::new();
    let mut cell_data: Vec<(i64, f64, f64, i64, i64, i64, i32)> = Vec::new();
    let mut cell_coords: HashMap<i64, (f64, f64)> = HashMap::new();

    for i in 0..10i64 {
        let zone_type = if i == 0 {
            ZONE_BIRTH
        } else if i == 9 {
            ZONE_DEATH
        } else {
            ZONE_COMMON
        };
        let forward_node = if i < 9 { i + 1 } else { -1 };
        let left_node = if i == 3 {
            14
        } else if i == 6 {
            24
        } else {
            -1
        };

        let x = i as f64;
        let y = 3.5;
        cells.push(pb::Cell {
            id: i,
            geom: Some(pb::Point { x, y }),
            zone_type,
            speed_limit: 1,
            left_node,
            forward_node,
            right_node: -1,
            meso_link_id: 0,
        });
        cell_data.push((i, x, y, forward_node, left_node, -1, zone_type));
        cell_coords.insert(i, (x, y));
    }

    for i in 0..10i64 {
        let cell_id = 10 + i;
        let zone_type = if i == 0 {
            ZONE_BIRTH
        } else if i == 9 {
            ZONE_DEATH
        } else {
            ZONE_COMMON
        };
        let forward_node = if i < 9 { cell_id + 1 } else { -1 };
        let right_node = if i == 3 { 4 } else { -1 };

        let x = 3.5;
        let y = i as f64;
        cells.push(pb::Cell {
            id: cell_id,
            geom: Some(pb::Point { x, y }),
            zone_type,
            speed_limit: 1,
            left_node: -1,
            forward_node,
            right_node,
            meso_link_id: 0,
        });
        cell_data.push((cell_id, x, y, forward_node, -1, right_node, zone_type));
        cell_coords.insert(cell_id, (x, y));
    }

    for i in 0..10i64 {
        let cell_id = 20 + i;
        let zone_type = if i == 0 {
            ZONE_BIRTH
        } else if i == 9 {
            ZONE_DEATH
        } else {
            ZONE_COMMON
        };
        let forward_node = if i < 9 { cell_id + 1 } else { -1 };
        let right_node = if i == 3 { 7 } else { -1 };

        let x = 6.5;
        let y = i as f64;
        cells.push(pb::Cell {
            id: cell_id,
            geom: Some(pb::Point { x, y }),
            zone_type,
            speed_limit: 1,
            left_node: -1,
            forward_node,
            right_node,
            meso_link_id: 0,
        });
        cell_data.push((cell_id, x, y, forward_node, -1, right_node, zone_type));
        cell_coords.insert(cell_id, (x, y));
    }

    let grid_request = pb::SessionGrid {
        session_id: Some(pb::UuiDv4 { value: sid.clone() }),
        data: cells,
    };
    let grid_stream = tokio_stream::once(grid_request);
    let mut grid_response = client.push_session_grid(grid_stream).await?.into_inner();
    while let Some(resp) = grid_response.next().await {
        let resp = resp?;
        eprintln!("grid push: code={} text={}", resp.code, resp.text);
    }

    let conflict_zones = vec![pb::ConflictZone {
        id: 1,
        source_x: 3,
        target_x: 4,
        source_y: 13,
        target_y: 14,
        conflict_winner: 3,
        conflict_type: 0,
    }];

    let cz_request = pb::SessionConflictZones {
        session_id: Some(pb::UuiDv4 { value: sid.clone() }),
        data: conflict_zones,
    };
    let cz_stream = tokio_stream::once(cz_request);
    let mut cz_response = client
        .push_session_conflict_zones(cz_stream)
        .await?
        .into_inner();
    while let Some(resp) = cz_response.next().await {
        let resp = resp?;
        eprintln!("conflict zones push: code={} text={}", resp.code, resp.text);
    }

    let tls = vec![pb::TrafficLight {
        id: 1,
        geom: Some(pb::Point { x: 7.0, y: 4.0 }),
        groups: vec![
            pb::Group {
                id: 100,
                label: "Group block H".to_string(),
                geom: vec![],
                cells: vec![6],
                signals: vec!["g".to_string(), "r".to_string()],
                movements: vec![],
                crosswalk_length: 0.0,
                r#type: 1,
            },
            pb::Group {
                id: 200,
                label: "Group block V2".to_string(),
                geom: vec![],
                cells: vec![23],
                signals: vec!["r".to_string(), "g".to_string()],
                movements: vec![],
                crosswalk_length: 0.0,
                r#type: 1,
            },
        ],
        times: vec![5, 5],
        signals_kinds: vec![],
    }];

    let mut tls_group_cells: HashMap<(i64, i64), Vec<i64>> = HashMap::new();
    for tl in &tls {
        for group in &tl.groups {
            tls_group_cells.insert((tl.id, group.id), group.cells.clone());
        }
    }

    let tls_request = pb::SessionTls {
        session_id: Some(pb::UuiDv4 { value: sid.clone() }),
        data: tls.clone(),
    };
    let tls_stream = tokio_stream::once(tls_request);
    let mut tls_response = client.push_session_tls(tls_stream).await?.into_inner();
    while let Some(resp) = tls_response.next().await {
        let resp = resp?;
        eprintln!("tls push: code={} text={}", resp.code, resp.text);
    }

    let trips = vec![
        pb::Trip {
            id: 1,
            trip_type: 2,
            from_node: 1,
            to_node: 9,
            initial_speed: 1,
            probability: 0.2,
            agent_type: 1,
            behaviour_type: 3,
            time: 0,
            start_time: 0,
            end_time: 0,
            relax_time: 0,
            transits: vec![],
        },
        pb::Trip {
            id: 2,
            trip_type: 2,
            from_node: 10,
            to_node: 19,
            initial_speed: 1,
            probability: 0.3,
            agent_type: 1,
            behaviour_type: 3,
            time: 0,
            start_time: 0,
            end_time: 0,
            relax_time: 0,
            transits: vec![],
        },
        pb::Trip {
            id: 3,
            trip_type: 2,
            from_node: 20,
            to_node: 29,
            initial_speed: 1,
            probability: 0.1,
            agent_type: 1,
            behaviour_type: 3,
            time: 0,
            start_time: 0,
            end_time: 0,
            relax_time: 0,
            transits: vec![],
        },
    ];

    let trip_request = pb::SessionTrip {
        session_id: Some(pb::UuiDv4 { value: sid.clone() }),
        data: trips,
    };
    let trip_stream = tokio_stream::once(trip_request);
    let mut trip_response = client.push_session_trip(trip_stream).await?.into_inner();
    while let Some(resp) = trip_response.next().await {
        let resp = resp?;
        eprintln!("trip push: code={} text={}", resp.code, resp.text);
    }

    let zone_str = |z: i32| -> &'static str {
        match z {
            1 => "birth",
            2 => "death",
            3 => "coordination",
            4 => "common",
            5 => "isolated",
            6 => "lane_for_bus",
            7 => "transit",
            8 => "crosswalk",
            _ => "undefined",
        }
    };

    println!("tl_id;x;y");
    for tl in &tls {
        if let Some(geom) = &tl.geom {
            println!("{};{:.5};{:.5}", tl.id, geom.x, geom.y);
        }
    }

    println!("tl_id;controlled_cell;x;y");
    for tl in &tls {
        for group in &tl.groups {
            for &cell_id in &group.cells {
                if let Some(&(x, y)) = cell_coords.get(&cell_id) {
                    println!("{};{};{:.5};{:.5}", tl.id, cell_id, x, y);
                }
            }
        }
    }

    println!("cell_id;x;y;forward_x;forward_y;connection_type;zone");
    for &(id, x, y, _, _, _, zone) in &cell_data {
        println!(
            "{};{:.5};{:.5};{:.5};{:.5};cell;{}",
            id,
            x,
            y,
            x,
            y,
            zone_str(zone)
        );
    }
    for &(id, x, y, fwd, left, right, _) in &cell_data {
        if fwd != -1 {
            if let Some(&(fx, fy)) = cell_coords.get(&fwd) {
                println!(
                    "{};{:.5};{:.5};{:.5};{:.5};forward;common",
                    id, x, y, fx, fy
                );
            }
        }
        if left != -1 {
            if let Some(&(lx, ly)) = cell_coords.get(&left) {
                println!("{};{:.5};{:.5};{:.5};{:.5};left;common", id, x, y, lx, ly);
            }
        }
        if right != -1 {
            if let Some(&(rx, ry)) = cell_coords.get(&right) {
                println!("{};{:.5};{:.5};{:.5};{:.5};right;common", id, x, y, rx, ry);
            }
        }
    }

    let rr_req = pb::RunAndRecordRequest {
        session_id: Some(pb::UuiDv4 { value: sid.clone() }),
        horizon_ticks: 50,
        batch_ticks: 20,
        filter: None,
    };
    let mut stream = client.run_and_record(rr_req).await?.into_inner();

    println!("\n=== Running 50 simulation steps ===\n");

    let mut tls_rows: Vec<String> = Vec::new();
    println!(
        "step;vehicle_id;vehicle_type;speed;bearing;intermediate_cells;cell;x;y;tail_cells;trip_id"
    );
    while let Some(resp) = stream.next().await {
        match resp?.payload {
            Some(pb::run_and_record_response::Payload::Metadata(m)) => {
                let cols = m
                    .schema
                    .map(|s| {
                        s.columns
                            .into_iter()
                            .map(|c| c.name)
                            .collect::<Vec<_>>()
                            .join(",")
                    })
                    .unwrap_or_default();
                eprintln!(
                    "format_version={} tick_seconds={} spawn_seed={} stochastic_seed={} columns=[{}]",
                    m.format_version, m.tick_seconds, m.spawn_seed, m.stochastic_seed, cols
                );
            }
            Some(pb::run_and_record_response::Payload::Batch(b)) => {
                decode_batch(&b.columns, &cell_coords, &tls_group_cells, &mut tls_rows);
            }
            Some(pb::run_and_record_response::Payload::Summary(s)) => {
                eprintln!(
                    "ticks={} rows={} raw_bytes={} completed={} lost={}",
                    s.total_ticks,
                    s.total_rows,
                    s.total_bytes,
                    s.vehicles_completed,
                    s.vehicles_lost
                );
            }
            None => {}
        }
    }

    println!("tl_step;tl_id;group_id;cell_id;x;y;signal");
    for row in &tls_rows {
        println!("{row}");
    }

    println!("\nSimulation complete!");
    Ok(())
}

fn decode_batch(
    blob: &[u8],
    cell_coords: &HashMap<i64, (f64, f64)>,
    tls_group_cells: &HashMap<(i64, i64), Vec<i64>>,
    tls_rows: &mut Vec<String>,
) {
    let responses = decode_record_batch(blob).expect("decode record batch");

    let vtype = |t: i32| match t {
        1 => "car",
        2 => "bus",
        3 => "taxi",
        4 => "pedestrian",
        5 => "truck",
        6 => "large_bus",
        _ => "undefined",
    };
    let join = |cells: &[i64]| -> String {
        cells
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",")
    };

    for resp in &responses {
        for v in &resp.vehicle_data {
            let (x, y) = cell_coords
                .get(&v.cell)
                .copied()
                .unwrap_or((f64::NAN, f64::NAN));
            println!(
                "{};{};{};{};{:.2};{};{};{:.2};{:.2};{};{}",
                resp.timestamp,
                v.vehicle_id,
                vtype(v.vehicle_type),
                v.speed,
                v.bearing,
                join(&v.intermediate_cells),
                v.cell,
                x,
                y,
                join(&v.tail_cells),
                v.trip_id,
            );
        }

        for tls in &resp.tls_data {
            let tl_id = tls.id;
            for group in &tls.groups {
                let group_id = group.id;
                if let Some(cells) = tls_group_cells.get(&(tl_id, group_id)) {
                    for &cell_id in cells {
                        if let Some(&(x, y)) = cell_coords.get(&cell_id) {
                            tls_rows.push(format!(
                                "{};{};{};{};{:.5};{:.5};{}",
                                resp.timestamp, tl_id, group_id, cell_id, x, y, group.signal
                            ));
                        }
                    }
                }
            }
        }
    }
}
