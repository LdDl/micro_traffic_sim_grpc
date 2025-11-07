package main

import (
	"context"
	"fmt"
	"os"
	"strings"

	microtraffic "github.com/LdDl/micro_traffic_sim_grpc/clients/go"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

func main() {
	// MT_SIM_ADDR can be: 127.0.0.1:50051 or http://127.0.0.1:50051
	raw := os.Getenv("MT_SIM_ADDR")
	if raw == "" {
		raw = "127.0.0.1:50051"
	}
	addr := raw
	if !strings.HasPrefix(addr, "http://") && !strings.HasPrefix(addr, "https://") {
		// For grpc.Dial we strip any scheme; tonic client in Rust adds http:// for Endpoint.
		// Here I just keep host:port form.
	}

	conn, err := grpc.Dial(addr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		panic(err)
	}
	defer conn.Close()

	cli := microtraffic.NewServiceClient(conn)

	// Create session (srid: 0 -> Euclidean, 4326 -> WGS84)
	newResp, err := cli.NewSession(context.Background(), &microtraffic.SessionReq{Srid: 0})
	if err != nil {
		panic(err)
	}
	if newResp.Id == nil {
		panic("empty session id")
	}
	sid := newResp.Id.Value

	fmt.Println("New session created:")
	fmt.Printf("  code: %d text: %s\n", newResp.Code, newResp.Text)
	fmt.Printf("  id:   %s\n", sid)

	// Info session
	infoResp, err := cli.InfoSession(context.Background(), &microtraffic.UUIDv4{Value: sid})
	if err != nil {
		panic(err)
	}
	fmt.Println("Info session:")
	fmt.Printf("  code: %d text: %s\n", infoResp.Code, infoResp.Text)
	if infoResp.Data != nil && infoResp.Data.Id != nil {
		fmt.Printf("  id:   %s\n", infoResp.Data.Id.Value)
	}
}
