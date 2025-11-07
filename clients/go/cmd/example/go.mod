module main

go 1.24.0

// Use the Go client module from the local path
// require github.com/LdDl/micro_traffic_sim_grpc/clients/go v0.0.0
replace github.com/LdDl/micro_traffic_sim_grpc/clients/go => ../../../go

require (
	github.com/LdDl/micro_traffic_sim_grpc/clients/go v0.0.0-00010101000000-000000000000
	google.golang.org/grpc v1.76.0
)

require (
	golang.org/x/net v0.46.0 // indirect
	golang.org/x/sys v0.37.0 // indirect
	golang.org/x/text v0.30.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20251103181224-f26f9409b101 // indirect
	google.golang.org/protobuf v1.36.10 // indirect
)
