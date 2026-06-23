use std::collections::HashMap;
use std::env;

use tokio_stream::StreamExt;
use tonic::transport::Channel;

use micro_traffic_sim::pb;
use micro_traffic_sim::pb::service_client::ServiceClient;

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

    let resp = client.new_session(pb::SessionReq { srid: 0 }).await?.into_inner();
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
        let left_node = if i == 3 { 14 } else if i == 6 { 24 } else { -1 };

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
    let mut cz_response = client.push_session_conflict_zones(cz_stream).await?.into_inner();
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
        println!("{};{:.5};{:.5};{:.5};{:.5};cell;{}", id, x, y, x, y, zone_str(zone));
    }
    for &(id, x, y, fwd, left, right, _) in &cell_data {
        if fwd != -1 {
            if let Some(&(fx, fy)) = cell_coords.get(&fwd) {
                println!("{};{:.5};{:.5};{:.5};{:.5};forward;common", id, x, y, fx, fy);
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

    println!("step;vehicle_id;vehicle_type;speed;bearing;intermediate_cells;cell;x;y;tail_cells;trip_id");
    while let Some(resp) = stream.next().await {
        match resp?.payload {
            Some(pb::run_and_record_response::Payload::Metadata(m)) => {
                let cols = m
                    .schema
                    .map(|s| s.columns.into_iter().map(|c| c.name).collect::<Vec<_>>().join(","))
                    .unwrap_or_default();
                eprintln!(
                    "format_version={} tick_seconds={} spawn_seed={} stochastic_seed={} columns=[{}]",
                    m.format_version, m.tick_seconds, m.spawn_seed, m.stochastic_seed, cols
                );
            }
            Some(pb::run_and_record_response::Payload::Batch(b)) => {
                decode_batch(&b.columns, &cell_coords);
            }
            Some(pb::run_and_record_response::Payload::Summary(s)) => {
                eprintln!(
                    "ticks={} rows={} raw_bytes={} completed={} lost={}",
                    s.total_ticks, s.total_rows, s.total_bytes, s.vehicles_completed, s.vehicles_lost
                );
            }
            None => {}
        }
    }

    println!("tl_step;tl_id;group_id;cell_id;x;y;signal");

    eprintln!("done");
    Ok(())
}

fn decode_batch(blob: &[u8], cell_coords: &HashMap<i64, (f64, f64)>) {
    let rd_u32 = |b: &[u8], o: &mut usize| -> u32 {
        let v = u32::from_le_bytes(b[*o..*o + 4].try_into().unwrap());
        *o += 4;
        v
    };
    let rd_u16 = |b: &[u8], o: &mut usize| -> u16 {
        let v = u16::from_le_bytes(b[*o..*o + 2].try_into().unwrap());
        *o += 2;
        v
    };
    let rd_i16 = |b: &[u8], o: &mut usize| -> i16 {
        let v = i16::from_le_bytes(b[*o..*o + 2].try_into().unwrap());
        *o += 2;
        v
    };

    let mut o = 0usize;
    o += 1;
    let tick_start = rd_u32(blob, &mut o);
    let tick_count = rd_u32(blob, &mut o) as usize;
    let r = rd_u32(blob, &mut o) as usize;

    let rows_per_tick: Vec<usize> = (0..tick_count).map(|_| rd_u32(blob, &mut o) as usize).collect();
    let veh_id: Vec<u32> = (0..r).map(|_| rd_u32(blob, &mut o)).collect();
    let cell: Vec<u32> = (0..r).map(|_| rd_u32(blob, &mut o)).collect();
    let agent_type = blob[o..o + r].to_vec();
    o += r;
    let angle: Vec<u16> = (0..r).map(|_| rd_u16(blob, &mut o)).collect();
    let speed: Vec<i16> = (0..r).map(|_| rd_i16(blob, &mut o)).collect();
    let trip: Vec<u32> = (0..r).map(|_| rd_u32(blob, &mut o)).collect();
    let ic_off: Vec<usize> = (0..r).map(|_| rd_u32(blob, &mut o) as usize).collect();
    let ic_total = ic_off.last().copied().unwrap_or(0);
    let ic_vals: Vec<u32> = (0..ic_total).map(|_| rd_u32(blob, &mut o)).collect();
    let tail_off: Vec<usize> = (0..r).map(|_| rd_u32(blob, &mut o) as usize).collect();
    let tail_total = tail_off.last().copied().unwrap_or(0);
    let tail_vals: Vec<u32> = (0..tail_total).map(|_| rd_u32(blob, &mut o)).collect();

    let vtype = |t: u8| match t {
        1 => "car",
        2 => "bus",
        3 => "taxi",
        4 => "pedestrian",
        5 => "truck",
        6 => "large_bus",
        _ => "undefined",
    };
    let slice = |off: &[usize], vals: &[u32], row: usize| -> String {
        let start = if row == 0 { 0 } else { off[row - 1] };
        vals[start..off[row]].iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",")
    };

    let mut row = 0usize;
    for (t, &n) in rows_per_tick.iter().enumerate() {
        let step = tick_start as usize + t;
        for _ in 0..n {
            let c = cell[row] as i64;
            let (x, y) = cell_coords.get(&c).copied().unwrap_or((f64::NAN, f64::NAN));
            println!(
                "{};{};{};{};{:.2};{};{};{:.2};{:.2};{};{}",
                step,
                veh_id[row],
                vtype(agent_type[row]),
                speed[row],
                angle[row] as f64 / 100.0,
                slice(&ic_off, &ic_vals, row),
                c,
                x,
                y,
                slice(&tail_off, &tail_vals, row),
                trip[row],
            );
            row += 1;
        }
    }
}
