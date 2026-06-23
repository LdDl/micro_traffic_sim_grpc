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

type CellData struct {
	ID          int64
	X, Y        float64
	ForwardNode int64
	LeftNode    int64
	RightNode   int64
	ZoneType    microtraffic.ZoneType
}

func main() {
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

	newResp, err := cli.NewSession(ctx, &microtraffic.SessionReq{Srid: 0})
	if err != nil {
		panic(err)
	}
	if newResp.Id == nil {
		panic("server returned empty session id")
	}
	sid := newResp.Id.Value
	fmt.Fprintf(os.Stderr, "session: %s\n", sid)

	var cells []*microtraffic.Cell
	var cellData []CellData
	cellCoords := make(map[int64][2]float64)

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
		fmt.Fprintf(os.Stderr, "grid push: code=%d text=%s\n", resp.Code, resp.Text)
	}

	conflictZones := []*microtraffic.ConflictZone{
		{
			Id:             1,
			SourceX:        3,
			TargetX:        4,
			SourceY:        13,
			TargetY:        14,
			ConflictWinner: microtraffic.ConflictWinnerType_CONFLICT_WINNER_SECOND,
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
		fmt.Fprintf(os.Stderr, "conflict zones push: code=%d text=%s\n", resp.Code, resp.Text)
	}

	tls := []*microtraffic.TrafficLight{
		{
			Id:   1,
			Geom: &microtraffic.Point{X: 7.0, Y: 4.0},
			Groups: []*microtraffic.Group{
				{
					Id:              100,
					Label:           "Group block H",
					Cells:           []int64{6},
					Signals:         []string{"g", "r"},
					Type:            microtraffic.GroupType_GROUP_TYPE_VEHICLE,
					CrosswalkLength: 0.0,
				},
				{
					Id:              200,
					Label:           "Group block V2",
					Cells:           []int64{23},
					Signals:         []string{"r", "g"},
					Type:            microtraffic.GroupType_GROUP_TYPE_VEHICLE,
					CrosswalkLength: 0.0,
				},
			},
			Times: []int64{5, 5},
		},
	}

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
		fmt.Fprintf(os.Stderr, "tls push: code=%d text=%s\n", resp.Code, resp.Text)
	}

	trips := []*microtraffic.Trip{
		{
			Id:            1,
			TripType:      microtraffic.TripType_TRIP_TYPE_RANDOM,
			FromNode:      1,
			ToNode:        9,
			InitialSpeed:  1,
			Probability:   0.2,
			AgentType:     microtraffic.AgentType_AGENT_TYPE_CAR,
			BehaviourType: microtraffic.BehaviourType_BEHAVIOUR_TYPE_COOPERATIVE,
		},
		{
			Id:            2,
			TripType:      microtraffic.TripType_TRIP_TYPE_RANDOM,
			FromNode:      10,
			ToNode:        19,
			InitialSpeed:  1,
			Probability:   0.3,
			AgentType:     microtraffic.AgentType_AGENT_TYPE_CAR,
			BehaviourType: microtraffic.BehaviourType_BEHAVIOUR_TYPE_COOPERATIVE,
		},
		{
			Id:            3,
			TripType:      microtraffic.TripType_TRIP_TYPE_RANDOM,
			FromNode:      20,
			ToNode:        29,
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
		fmt.Fprintf(os.Stderr, "trip push: code=%d text=%s\n", resp.Code, resp.Text)
	}

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

	fmt.Println("tl_id;x;y")
	for _, tl := range tls {
		if tl.Geom != nil {
			fmt.Printf("%d;%.5f;%.5f\n", tl.Id, tl.Geom.X, tl.Geom.Y)
		}
	}

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

	fmt.Println("cell_id;x;y;forward_x;forward_y;connection_type;zone")
	for _, cd := range cellData {
		fmt.Printf("%d;%.5f;%.5f;%.5f;%.5f;cell;%s\n", cd.ID, cd.X, cd.Y, cd.X, cd.Y, zoneStr(cd.ZoneType))
	}
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

	stream, err := cli.RunAndRecord(ctx, &microtraffic.RunAndRecordRequest{
		SessionId:    &microtraffic.UUIDv4{Value: sid},
		HorizonTicks: 50,
		BatchTicks:   20,
	})
	if err != nil {
		panic(err)
	}

	fmt.Print("\n=== Running 50 simulation steps ===\n\n")

	var tlsRows []string
	fmt.Println("step;vehicle_id;vehicle_type;speed;bearing;intermediate_cells;cell;x;y;tail_cells;trip_id")
	for {
		resp, err := stream.Recv()
		if err == io.EOF {
			break
		}
		if err != nil {
			panic(err)
		}
		if m := resp.GetMetadata(); m != nil {
			var cols []string
			if m.Schema != nil {
				for _, c := range m.Schema.Columns {
					cols = append(cols, c.Name)
				}
			}
			fmt.Fprintf(os.Stderr, "format_version=%d tick_seconds=%g spawn_seed=%d stochastic_seed=%d columns=[%s]\n",
				m.FormatVersion, m.TickSeconds, m.SpawnSeed, m.StochasticSeed, strings.Join(cols, ","))
		} else if b := resp.GetBatch(); b != nil {
			tlsRows = append(tlsRows, decodeBatch(b.Columns, cellCoords, tlsGroupCells)...)
		} else if s := resp.GetSummary(); s != nil {
			fmt.Fprintf(os.Stderr, "ticks=%d rows=%d raw_bytes=%d completed=%d lost=%d\n",
				s.TotalTicks, s.TotalRows, s.TotalBytes, s.VehiclesCompleted, s.VehiclesLost)
		}
	}

	fmt.Println("tl_step;tl_id;group_id;cell_id;x;y;signal")
	for _, row := range tlsRows {
		fmt.Println(row)
	}

	fmt.Println("\nSimulation complete!")
}

func decodeBatch(blob []byte, cellCoords map[int64][2]float64, tlsGroupCells map[[2]int64][]int64) []string {
	responses, err := microtraffic.DecodeRecordBatch(blob)
	if err != nil {
		panic(err)
	}

	cellsStr := func(vals []int64) string {
		parts := make([]string, 0, len(vals))
		for _, v := range vals {
			parts = append(parts, fmt.Sprintf("%d", v))
		}
		return strings.Join(parts, ",")
	}

	var tlsRows []string
	for _, resp := range responses {
		timestamp := resp.Timestamp

		for _, v := range resp.VehicleData {
			c := v.Cell
			x, y := math.NaN(), math.NaN()
			if xy, ok := cellCoords[c]; ok {
				x, y = xy[0], xy[1]
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
			case microtraffic.AgentType_AGENT_TYPE_TRUCK:
				vehicleType = "truck"
			case microtraffic.AgentType_AGENT_TYPE_LARGE_BUS:
				vehicleType = "large_bus"
			}

			fmt.Printf("%d;%d;%s;%d;%.2f;%s;%d;%.2f;%.2f;%s;%d\n",
				timestamp, v.VehicleId, vehicleType, v.Speed, v.Bearing,
				cellsStr(v.IntermediateCells), c, x, y, cellsStr(v.TailCells), v.TripId)
		}

		for _, tlsState := range resp.TlsData {
			for _, group := range tlsState.Groups {
				if cells, ok := tlsGroupCells[[2]int64{tlsState.Id, group.Id}]; ok {
					for _, cellID := range cells {
						if xy, ok := cellCoords[cellID]; ok {
							tlsRows = append(tlsRows, fmt.Sprintf("%d;%d;%d;%d;%.5f;%.5f;%s",
								timestamp, tlsState.Id, group.Id, cellID, xy[0], xy[1], group.Signal))
						}
					}
				}
			}
		}
	}
	return tlsRows
}
