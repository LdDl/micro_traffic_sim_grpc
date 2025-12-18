package main

import (
	"context"
	"fmt"
	"io"
	"math"
	"os"
	"strings"

	microtraffic "github.com/LdDl/micro_traffic_sim_grpc/clients/go"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

// CellData stores cell information for gnuplot output
type CellData struct {
	ID          int64
	X, Y        float64
	ForwardNode int64
	LeftNode    int64
	RightNode   int64
	ZoneType    microtraffic.ZoneType
}

// VehicleStateRecord stores vehicle state for output
type VehicleStateRecord struct {
	Step              int64
	VehicleID         int64
	VehicleType       string
	Speed             int64
	Bearing           float64
	IntermediateCells string
	Cell              int64
	X, Y              float64
	TailCells         string
	TripID            int64
}

// TLSStateRecord stores TLS state for output
type TLSStateRecord struct {
	Step    int64
	TLID    int64
	GroupID int64
	CellID  int64
	X, Y    float64
	Signal  string
}

func main() {
	// MT_SIM_ADDR can be: 127.0.0.1:50051 or http://127.0.0.1:50051
	raw := os.Getenv("MT_SIM_ADDR")
	if raw == "" {
		raw = "127.0.0.1:50051"
	}
	addr := strings.TrimPrefix(strings.TrimPrefix(raw, "http://"), "https://")

	conn, err := grpc.Dial(addr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		panic(err)
	}
	defer conn.Close()

	cli := microtraffic.NewServiceClient(conn)
	ctx := context.Background()

	// ==============================================================
	// STEP 1: CREATE SESSION
	// ==============================================================
	newResp, err := cli.NewSession(ctx, &microtraffic.SessionReq{Srid: 0}) // Euclidean coordinates
	if err != nil {
		panic(err)
	}
	if newResp.Id == nil {
		panic("server returned empty session id")
	}
	sid := newResp.Id.Value
	fmt.Printf("Session created: %s\n", sid)

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

	var cells []*microtraffic.Cell
	var cellData []CellData
	cellCoords := make(map[int64][2]float64)

	// HORIZONTAL ROAD (cells 0-9)
	for i := int64(0); i < 10; i++ {
		zoneType := microtraffic.ZoneType_ZONE_TYPE_COMMON
		if i == 0 {
			zoneType = microtraffic.ZoneType_ZONE_TYPE_BIRTH
		} else if i == 9 {
			zoneType = microtraffic.ZoneType_ZONE_TYPE_DEATH
		}
		forwardNode := int64(-1)
		if i < 9 {
			forwardNode = i + 1
		}
		leftNode := int64(-1)
		if i == 3 {
			leftNode = 14
		} else if i == 6 {
			leftNode = 24
		}

		x := float64(i)
		y := 3.5
		cells = append(cells, &microtraffic.Cell{
			Id:          i,
			Geom:        &microtraffic.Point{X: x, Y: y},
			ZoneType:    zoneType,
			SpeedLimit:  1,
			LeftNode:    leftNode,
			ForwardNode: forwardNode,
			RightNode:   -1,
			MesoLinkId:  0,
		})
		cellData = append(cellData, CellData{
			ID: i, X: x, Y: y,
			ForwardNode: forwardNode, LeftNode: leftNode, RightNode: -1,
			ZoneType: zoneType,
		})
		cellCoords[i] = [2]float64{x, y}
	}

	// VERTICAL ROAD 1 (cells 10-19, x=3.5)
	for i := int64(0); i < 10; i++ {
		cellID := 10 + i
		zoneType := microtraffic.ZoneType_ZONE_TYPE_COMMON
		if i == 0 {
			zoneType = microtraffic.ZoneType_ZONE_TYPE_BIRTH
		} else if i == 9 {
			zoneType = microtraffic.ZoneType_ZONE_TYPE_DEATH
		}
		forwardNode := int64(-1)
		if i < 9 {
			forwardNode = cellID + 1
		}
		rightNode := int64(-1)
		if i == 3 {
			rightNode = 4
		}

		x := 3.5
		y := float64(i)
		cells = append(cells, &microtraffic.Cell{
			Id:          cellID,
			Geom:        &microtraffic.Point{X: x, Y: y},
			ZoneType:    zoneType,
			SpeedLimit:  1,
			LeftNode:    -1,
			ForwardNode: forwardNode,
			RightNode:   rightNode,
			MesoLinkId:  0,
		})
		cellData = append(cellData, CellData{
			ID: cellID, X: x, Y: y,
			ForwardNode: forwardNode, LeftNode: -1, RightNode: rightNode,
			ZoneType: zoneType,
		})
		cellCoords[cellID] = [2]float64{x, y}
	}

	// VERTICAL ROAD 2 (cells 20-29, x=6.5)
	for i := int64(0); i < 10; i++ {
		cellID := 20 + i
		zoneType := microtraffic.ZoneType_ZONE_TYPE_COMMON
		if i == 0 {
			zoneType = microtraffic.ZoneType_ZONE_TYPE_BIRTH
		} else if i == 9 {
			zoneType = microtraffic.ZoneType_ZONE_TYPE_DEATH
		}
		forwardNode := int64(-1)
		if i < 9 {
			forwardNode = cellID + 1
		}
		rightNode := int64(-1)
		if i == 3 {
			rightNode = 7
		}

		x := 6.5
		y := float64(i)
		cells = append(cells, &microtraffic.Cell{
			Id:          cellID,
			Geom:        &microtraffic.Point{X: x, Y: y},
			ZoneType:    zoneType,
			SpeedLimit:  1,
			LeftNode:    -1,
			ForwardNode: forwardNode,
			RightNode:   rightNode,
			MesoLinkId:  0,
		})
		cellData = append(cellData, CellData{
			ID: cellID, X: x, Y: y,
			ForwardNode: forwardNode, LeftNode: -1, RightNode: rightNode,
			ZoneType: zoneType,
		})
		cellCoords[cellID] = [2]float64{x, y}
	}

	// Push grid via streaming
	gridStream, err := cli.PushSessionGrid(ctx)
	if err != nil {
		panic(err)
	}
	err = gridStream.Send(&microtraffic.SessionGrid{
		SessionId: &microtraffic.UUIDv4{Value: sid},
		Data:      cells,
	})
	if err != nil {
		panic(err)
	}
	gridStream.CloseSend()
	for {
		resp, err := gridStream.Recv()
		if err == io.EOF {
			break
		}
		if err != nil {
			panic(err)
		}
		fmt.Printf("Grid push response: code=%d text=%s\n", resp.Code, resp.Text)
	}

	// ==============================================================
	// STEP 3: PUSH CONFLICT ZONES
	// ==============================================================
	conflictZones := []*microtraffic.ConflictZone{
		{
			Id:             1,
			SourceX:        3,  // H cell before intersection
			TargetX:        4,  // H cell after intersection
			SourceY:        13, // V1 cell before intersection
			TargetY:        14, // V1 cell after intersection
			ConflictWinner: microtraffic.ConflictWinnerType_CONFLICT_WINNER_SECOND, // V1 has priority
			ConflictType:   microtraffic.ConflictZoneType_CONFLICT_ZONE_TYPE_UNDEFINED,
		},
	}

	czStream, err := cli.PushSessionConflictZones(ctx)
	if err != nil {
		panic(err)
	}
	err = czStream.Send(&microtraffic.SessionConflictZones{
		SessionId: &microtraffic.UUIDv4{Value: sid},
		Data:      conflictZones,
	})
	if err != nil {
		panic(err)
	}
	czStream.CloseSend()
	for {
		resp, err := czStream.Recv()
		if err == io.EOF {
			break
		}
		if err != nil {
			panic(err)
		}
		fmt.Printf("Conflict zones push response: code=%d text=%s\n", resp.Code, resp.Text)
	}

	// ==============================================================
	// STEP 4: PUSH TRAFFIC LIGHTS
	// ==============================================================
	tls := []*microtraffic.TrafficLight{
		{
			Id:   1,
			Geom: &microtraffic.Point{X: 7.0, Y: 4.0},
			Groups: []*microtraffic.Group{
				{
					Id:              100,
					Label:           "Group block H",
					Cells:           []int64{6},
					Signals:         []string{"g", "r"}, // Green, Red
					Type:            microtraffic.GroupType_GROUP_TYPE_VEHICLE,
					CrosswalkLength: 0.0,
				},
				{
					Id:              200,
					Label:           "Group block V2",
					Cells:           []int64{23},
					Signals:         []string{"r", "g"}, // Red, Green
					Type:            microtraffic.GroupType_GROUP_TYPE_VEHICLE,
					CrosswalkLength: 0.0,
				},
			},
			Times: []int64{5, 5}, // 5s green, 5s red
		},
	}

	// Store TLS group cells for output later: (tl_id, group_id) -> []cell_id
	tlsGroupCells := make(map[[2]int64][]int64)
	for _, tl := range tls {
		for _, group := range tl.Groups {
			tlsGroupCells[[2]int64{tl.Id, group.Id}] = group.Cells
		}
	}

	tlsStream, err := cli.PushSessionTLS(ctx)
	if err != nil {
		panic(err)
	}
	err = tlsStream.Send(&microtraffic.SessionTLS{
		SessionId: &microtraffic.UUIDv4{Value: sid},
		Data:      tls,
	})
	if err != nil {
		panic(err)
	}
	tlsStream.CloseSend()
	for {
		resp, err := tlsStream.Recv()
		if err == io.EOF {
			break
		}
		if err != nil {
			panic(err)
		}
		fmt.Printf("TLS push response: code=%d text=%s\n", resp.Code, resp.Text)
	}

	// ==============================================================
	// STEP 5: PUSH TRIPS (vehicle generators)
	// ==============================================================
	trips := []*microtraffic.Trip{
		{
			Id:            1,
			TripType:      microtraffic.TripType_TRIP_TYPE_RANDOM,
			FromNode:      1, // H birth (cell 1 since 0 is occupied)
			ToNode:        9, // H death
			InitialSpeed:  1,
			Probability:   0.2,
			AgentType:     microtraffic.AgentType_AGENT_TYPE_CAR,
			BehaviourType: microtraffic.BehaviourType_BEHAVIOUR_TYPE_COOPERATIVE,
		},
		{
			Id:            2,
			TripType:      microtraffic.TripType_TRIP_TYPE_RANDOM,
			FromNode:      10, // V1 birth
			ToNode:        19, // V1 death
			InitialSpeed:  1,
			Probability:   0.3,
			AgentType:     microtraffic.AgentType_AGENT_TYPE_CAR,
			BehaviourType: microtraffic.BehaviourType_BEHAVIOUR_TYPE_COOPERATIVE,
		},
		{
			Id:            3,
			TripType:      microtraffic.TripType_TRIP_TYPE_RANDOM,
			FromNode:      20, // V2 birth
			ToNode:        29, // V2 death
			InitialSpeed:  1,
			Probability:   0.1,
			AgentType:     microtraffic.AgentType_AGENT_TYPE_CAR,
			BehaviourType: microtraffic.BehaviourType_BEHAVIOUR_TYPE_COOPERATIVE,
		},
	}

	tripStream, err := cli.PushSessionTrip(ctx)
	if err != nil {
		panic(err)
	}
	err = tripStream.Send(&microtraffic.SessionTrip{
		SessionId: &microtraffic.UUIDv4{Value: sid},
		Data:      trips,
	})
	if err != nil {
		panic(err)
	}
	tripStream.CloseSend()
	for {
		resp, err := tripStream.Recv()
		if err == io.EOF {
			break
		}
		if err != nil {
			panic(err)
		}
		fmt.Printf("Trip push response: code=%d text=%s\n", resp.Code, resp.Text)
	}

	// ==============================================================
	// STEP 6: PRINT GRID/TLS METADATA FOR GNUPLOT
	// ==============================================================
	zoneStr := func(z microtraffic.ZoneType) string {
		switch z {
		case microtraffic.ZoneType_ZONE_TYPE_BIRTH:
			return "birth"
		case microtraffic.ZoneType_ZONE_TYPE_DEATH:
			return "death"
		case microtraffic.ZoneType_ZONE_TYPE_COORDINATION:
			return "coordination"
		case microtraffic.ZoneType_ZONE_TYPE_COMMON:
			return "common"
		case microtraffic.ZoneType_ZONE_TYPE_ISOLATED:
			return "isolated"
		case microtraffic.ZoneType_ZONE_TYPE_LANE_FOR_BUS:
			return "lane_for_bus"
		case microtraffic.ZoneType_ZONE_TYPE_TRANSIT:
			return "transit"
		case microtraffic.ZoneType_ZONE_TYPE_CROSSWALK:
			return "crosswalk"
		default:
			return "undefined"
		}
	}

	// Print TLS positions
	fmt.Println("tl_id;x;y")
	for _, tl := range tls {
		if tl.Geom != nil {
			fmt.Printf("%d;%.5f;%.5f\n", tl.Id, tl.Geom.X, tl.Geom.Y)
		}
	}

	// Print TLS controlled cells
	fmt.Println("tl_id;controlled_cell;x;y")
	for _, tl := range tls {
		for _, group := range tl.Groups {
			for _, cellID := range group.Cells {
				if coords, ok := cellCoords[cellID]; ok {
					fmt.Printf("%d;%d;%.5f;%.5f\n", tl.Id, cellID, coords[0], coords[1])
				}
			}
		}
	}

	// Print grid cells with connections
	fmt.Println("cell_id;x;y;forward_x;forward_y;connection_type;zone")
	// First print all cells
	for _, cd := range cellData {
		fmt.Printf("%d;%.5f;%.5f;%.5f;%.5f;cell;%s\n", cd.ID, cd.X, cd.Y, cd.X, cd.Y, zoneStr(cd.ZoneType))
	}
	// Then print connections (arrows)
	for _, cd := range cellData {
		if cd.ForwardNode != -1 {
			if fwd, ok := cellCoords[cd.ForwardNode]; ok {
				fmt.Printf("%d;%.5f;%.5f;%.5f;%.5f;forward;common\n", cd.ID, cd.X, cd.Y, fwd[0], fwd[1])
			}
		}
		if cd.LeftNode != -1 {
			if left, ok := cellCoords[cd.LeftNode]; ok {
				fmt.Printf("%d;%.5f;%.5f;%.5f;%.5f;left;common\n", cd.ID, cd.X, cd.Y, left[0], left[1])
			}
		}
		if cd.RightNode != -1 {
			if right, ok := cellCoords[cd.RightNode]; ok {
				fmt.Printf("%d;%.5f;%.5f;%.5f;%.5f;right;common\n", cd.ID, cd.X, cd.Y, right[0], right[1])
			}
		}
	}

	// ==============================================================
	// STEP 7: RUN SIMULATION
	// ==============================================================
	stepsNum := 50
	fmt.Printf("\n=== Running %d simulation steps ===\n\n", stepsNum)

	var vehicleStates []VehicleStateRecord
	var tlsStates []TLSStateRecord

	stepStream, err := cli.SimulationStepSession(ctx)
	if err != nil {
		panic(err)
	}

	// Send all step requests
	for i := 0; i < stepsNum; i++ {
		err = stepStream.Send(&microtraffic.SessionStep{
			SessionId: &microtraffic.UUIDv4{Value: sid},
		})
		if err != nil {
			panic(err)
		}
	}
	stepStream.CloseSend()

	// Receive responses
	for {
		resp, err := stepStream.Recv()
		if err == io.EOF {
			break
		}
		if err != nil {
			panic(err)
		}

		timestamp := resp.Timestamp

		// Collect vehicle states
		for _, v := range resp.VehicleData {
			x, y := math.NaN(), math.NaN()
			if v.Point != nil {
				x, y = v.Point.X, v.Point.Y
			}

			var intermediateCells []string
			for _, c := range v.IntermediateCells {
				intermediateCells = append(intermediateCells, fmt.Sprintf("%d", c))
			}

			var tailCells []string
			for _, c := range v.TailCells {
				tailCells = append(tailCells, fmt.Sprintf("%d", c))
			}

			vehicleType := "undefined"
			switch v.VehicleType {
			case microtraffic.AgentType_AGENT_TYPE_CAR:
				vehicleType = "car"
			case microtraffic.AgentType_AGENT_TYPE_BUS:
				vehicleType = "bus"
			case microtraffic.AgentType_AGENT_TYPE_TAXI:
				vehicleType = "taxi"
			case microtraffic.AgentType_AGENT_TYPE_PEDESTRIAN:
				vehicleType = "pedestrian"
			}

			vehicleStates = append(vehicleStates, VehicleStateRecord{
				Step:              timestamp,
				VehicleID:         v.VehicleId,
				VehicleType:       vehicleType,
				Speed:             v.Speed,
				Bearing:           v.Bearing,
				IntermediateCells: strings.Join(intermediateCells, ","),
				Cell:              v.Cell,
				X:                 x,
				Y:                 y,
				TailCells:         strings.Join(tailCells, ","),
				TripID:            v.TripId,
			})
		}

		// Collect TLS states (expand to per-cell)
		for _, tlsState := range resp.TlsData {
			for _, group := range tlsState.Groups {
				if cellIDs, ok := tlsGroupCells[[2]int64{tlsState.Id, group.Id}]; ok {
					for _, cellID := range cellIDs {
						x, y := 0.0, 0.0
						if coords, ok := cellCoords[cellID]; ok {
							x, y = coords[0], coords[1]
						}
						tlsStates = append(tlsStates, TLSStateRecord{
							Step:    timestamp,
							TLID:    tlsState.Id,
							GroupID: group.Id,
							CellID:  cellID,
							X:       x,
							Y:       y,
							Signal:  group.Signal,
						})
					}
				}
			}
		}
	}

	// Print vehicle states
	fmt.Println("step;vehicle_id;vehicle_type;speed;bearing;intermediate_cells;cell;x;y;tail_cells;trip_id")
	for _, vs := range vehicleStates {
		fmt.Printf("%d;%d;%s;%d;%.2f;%s;%d;%.2f;%.2f;%s;%d\n",
			vs.Step, vs.VehicleID, vs.VehicleType, vs.Speed, vs.Bearing,
			vs.IntermediateCells, vs.Cell, vs.X, vs.Y, vs.TailCells, vs.TripID)
	}

	// Print TLS states
	fmt.Println("tl_step;tl_id;group_id;cell_id;x;y;signal")
	for _, ts := range tlsStates {
		fmt.Printf("%d;%d;%d;%d;%.5f;%.5f;%s\n",
			ts.Step, ts.TLID, ts.GroupID, ts.CellID, ts.X, ts.Y, ts.Signal)
	}

	fmt.Println("\nSimulation complete!")
}
