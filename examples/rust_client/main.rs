use std::collections::HashMap;
use std::env;
use tokio_stream::StreamExt;
use tonic::transport::Channel;

use micro_traffic_sim::pb;
use micro_traffic_sim::pb::service_client::ServiceClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Server address (override with MT_SIM_ADDR, e.g. http://127.0.0.1:50051)
    let raw = env::var("MT_SIM_ADDR").unwrap_or_else(|_| "127.0.0.1:50051".to_string());
    let addr = if raw.starts_with("http://") || raw.starts_with("https://") {
        raw
    } else {
        format!("http://{raw}")
    };

    // Connect
    let channel = Channel::from_shared(addr.clone())?
        .connect()
        .await?;
    let mut client = ServiceClient::new(channel);

    // ==============================================================
    // STEP 1: CREATE SESSION
    // ==============================================================
    let req = pb::SessionReq { srid: 0 }; // Euclidean coordinates
    let resp = client.new_session(req).await?.into_inner();
    let sid = resp
        .id
        .as_ref()
        .map(|x| x.value.clone())
        .ok_or("server returned empty session id")?;
    println!("Session created: {}", sid);

    // ==============================================================
    // STEP 2: PUSH GRID CELLS
    // ==============================================================
    // Road layout:
    //        V1 (vertical 1)    V2 (vertical 2)
    //          |                 |
    //    H ----+-----------------+---- H (horizontal)
    //          |                 |
    // Horizontal road cells: 0-9 (y=3.5, x=0..9)
    // Vertical road 1 cells: 10-19 (y=0..9, x=3.5)
    // Vertical road 2 cells: 20-29 (y=0..9, x=6.5)

    let mut cells: Vec<pb::Cell> = Vec::new();
    // Store cell data for gnuplot output
    // (id, x, y, forward_node, left_node, right_node, zone_type)
    let mut cell_data: Vec<(i64, f64, f64, i64, i64, i64, i32)> = Vec::new();
    // Store cell coordinates for TLS output later
    let mut cell_coords: HashMap<i64, (f64, f64)> = HashMap::new();

    // ZoneType values: Birth=1, Death=2, Common=4
    const ZONE_BIRTH: i32 = 1;
    const ZONE_DEATH: i32 = 2;
    const ZONE_COMMON: i32 = 4;

    // HORIZONTAL ROAD (cells 0-9)
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

    // VERTICAL ROAD 1 (cells 10-19, x=3.5)
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

    // VERTICAL ROAD 2 (cells 20-29, x=6.5)
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

    // Push grid via streaming
    let grid_request = pb::SessionGrid {
        session_id: Some(pb::UuiDv4 { value: sid.clone() }),
        data: cells,
    };
    let grid_stream = tokio_stream::once(grid_request);
    let mut grid_response = client.push_session_grid(grid_stream).await?.into_inner();
    while let Some(resp) = grid_response.next().await {
        let resp = resp?;
        println!("Grid push response: code={} text={}", resp.code, resp.text);
    }

    // ==============================================================
    // STEP 3: PUSH CONFLICT ZONES
    // ==============================================================
    let conflict_zones = vec![pb::ConflictZone {
        id: 1,
        source_x: 3,  // H cell before intersection
        target_x: 4,  // H cell after intersection
        source_y: 13, // V1 cell before intersection
        target_y: 14, // V1 cell after intersection
        conflict_winner: 3, // CONFLICT_WINNER_SECOND = V1 has priority
        conflict_type: 0,   // CONFLICT_ZONE_TYPE_UNDEFINED
    }];

    let cz_request = pb::SessionConflictZones {
        session_id: Some(pb::UuiDv4 { value: sid.clone() }),
        data: conflict_zones,
    };
    let cz_stream = tokio_stream::once(cz_request);
    let mut cz_response = client.push_session_conflict_zones(cz_stream).await?.into_inner();
    while let Some(resp) = cz_response.next().await {
        let resp = resp?;
        println!("Conflict zones push response: code={} text={}", resp.code, resp.text);
    }

    // ==============================================================
    // STEP 4: PUSH TRAFFIC LIGHTS
    // ==============================================================
    // GroupType: Vehicle=1
    let tls = vec![pb::TrafficLight {
        id: 1,
        geom: Some(pb::Point { x: 7.0, y: 4.0 }),
        groups: vec![
            pb::Group {
                id: 100,
                label: "Group block H".to_string(),
                geom: vec![],
                cells: vec![6],
                signals: vec!["g".to_string(), "r".to_string()], // Green, Red
                movements: vec![],
                crosswalk_length: 0.0,
                r#type: 1, // GROUP_TYPE_VEHICLE
            },
            pb::Group {
                id: 200,
                label: "Group block V2".to_string(),
                geom: vec![],
                cells: vec![23],
                signals: vec!["r".to_string(), "g".to_string()], // Red, Green
                movements: vec![],
                crosswalk_length: 0.0,
                r#type: 1, // GROUP_TYPE_VEHICLE
            },
        ],
        times: vec![5, 5], // 5s green, 5s red
        signals_kinds: vec![],
    }];

    // Store TLS group cells for output later: (tl_id, group_id) -> Vec<cell_id>
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
        println!("TLS push response: code={} text={}", resp.code, resp.text);
    }

    // ==============================================================
    // STEP 5: PUSH TRIPS (vehicle generators)
    // ==============================================================
    // TripType: Random=2, AgentType: Car=1, BehaviourType: Cooperative=3
    let trips = vec![
        pb::Trip {
            id: 1,
            trip_type: 2, // TRIP_TYPE_RANDOM
            from_node: 1,  // H birth (but we use cell 1 since 0 is occupied)
            to_node: 9,    // H death
            initial_speed: 1,
            probability: 0.2,
            agent_type: 1,        // AGENT_TYPE_CAR
            behaviour_type: 3,    // BEHAVIOUR_TYPE_COOPERATIVE
            time: 0,
            start_time: 0,
            end_time: 0,
            relax_time: 0,
            transits: vec![],
        },
        pb::Trip {
            id: 2,
            trip_type: 2, // TRIP_TYPE_RANDOM
            from_node: 10, // V1 birth
            to_node: 19,   // V1 death
            initial_speed: 1,
            probability: 0.3,
            agent_type: 1,        // AGENT_TYPE_CAR
            behaviour_type: 3,    // BEHAVIOUR_TYPE_COOPERATIVE
            time: 0,
            start_time: 0,
            end_time: 0,
            relax_time: 0,
            transits: vec![],
        },
        pb::Trip {
            id: 3,
            trip_type: 2, // TRIP_TYPE_RANDOM
            from_node: 20, // V2 birth
            to_node: 29,   // V2 death
            initial_speed: 1,
            probability: 0.1,
            agent_type: 1,        // AGENT_TYPE_CAR
            behaviour_type: 3,    // BEHAVIOUR_TYPE_COOPERATIVE
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
        println!("Trip push response: code={} text={}", resp.code, resp.text);
    }

    // ==============================================================
    // STEP 6: PRINT GRID/TLS METADATA FOR GNUPLOT
    // ==============================================================
    // Helper to convert zone_type to string
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

    // Print TLS positions
    println!("tl_id;x;y");
    for tl in &tls {
        if let Some(geom) = &tl.geom {
            println!("{};{:.5};{:.5}", tl.id, geom.x, geom.y);
        }
    }

    // Print TLS controlled cells
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

    // Print grid cells with connections
    println!("cell_id;x;y;forward_x;forward_y;connection_type;zone");
    // First print all cells (for gnuplot to show them even without connections)
    for &(id, x, y, _, _, _, zone) in &cell_data {
        println!("{};{:.5};{:.5};{:.5};{:.5};cell;{}", id, x, y, x, y, zone_str(zone));
    }
    // Then print connections (arrows)
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

    // ==============================================================
    // STEP 7: RUN SIMULATION
    // ==============================================================
    let steps_num = 50;
    println!("\n=== Running {} simulation steps ===\n", steps_num);

    // Collect states for printing at the end
    let mut vehicle_states: Vec<(i64, i64, &'static str, i64, f64, String, i64, f64, f64, String, i64)> = Vec::new();
    // TLS states: (step, tl_id, group_id, cell_id, x, y, signal)
    let mut tls_states: Vec<(i64, i64, i64, i64, f64, f64, String)> = Vec::new();

    // Create step requests
    let step_requests: Vec<pb::SessionStep> = (0..steps_num)
        .map(|_| pb::SessionStep {
            session_id: Some(pb::UuiDv4 { value: sid.clone() }),
        })
        .collect();

    let step_stream = tokio_stream::iter(step_requests);
    let mut step_response = client.simulation_step_session(step_stream).await?.into_inner();

    while let Some(resp) = step_response.next().await {
        let resp = resp?;
        let timestamp = resp.timestamp;

        // Collect vehicle states
        for v in &resp.vehicle_data {
            let (x, y) = if let Some(pt) = &v.point {
                (pt.x, pt.y)
            } else {
                (f64::NAN, f64::NAN)
            };
            let intermediate_cells = v.intermediate_cells
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let tail_cells = v.tail_cells
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let vehicle_type = match v.vehicle_type {
                1 => "car",
                2 => "bus",
                3 => "taxi",
                4 => "pedestrian",
                _ => "undefined",
            };
            vehicle_states.push((
                timestamp,
                v.vehicle_id,
                vehicle_type,
                v.speed,
                v.bearing,
                intermediate_cells,
                v.cell,
                x,
                y,
                tail_cells,
                v.trip_id,
            ));
        }

        // Collect TLS states (expand to per-cell)
        for tls in &resp.tls_data {
            for group in &tls.groups {
                // Look up cells for this group
                if let Some(cells) = tls_group_cells.get(&(tls.id, group.id)) {
                    for &cell_id in cells {
                        let (x, y) = cell_coords.get(&cell_id).copied().unwrap_or((0.0, 0.0));
                        tls_states.push((timestamp, tls.id, group.id, cell_id, x, y, group.signal.clone()));
                    }
                }
            }
        }
    }

    // Print vehicle states
    println!("step;vehicle_id;vehicle_type;speed;bearing;intermediate_cells;cell;x;y;tail_cells;trip_id");
    for (step, vehicle_id, vehicle_type, speed, bearing, intermediate_cells, cell, x, y, tail_cells, trip_id) in &vehicle_states {
        println!(
            "{};{};{};{};{:.2};{};{};{:.2};{:.2};{};{}",
            step, vehicle_id, vehicle_type, speed, bearing, intermediate_cells, cell, x, y, tail_cells, trip_id
        );
    }

    // Print TLS states
    println!("tl_step;tl_id;group_id;cell_id;x;y;signal");
    for (step, tl_id, group_id, cell_id, x, y, signal) in &tls_states {
        println!("{};{};{};{};{:.5};{:.5};{}", step, tl_id, group_id, cell_id, x, y, signal);
    }

    println!("\nSimulation complete!");
    Ok(())
}
