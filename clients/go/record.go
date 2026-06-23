package microtraffic

import (
	"encoding/binary"
	"fmt"
)

// signalChar maps the on-wire signal code to its string representation
// (0=undefined 1=r 2=y 3=g 4=G 5=s 6=u 7=o 8=O).
func signalChar(c byte) string {
	switch c {
	case 1:
		return "r"
	case 2:
		return "y"
	case 3:
		return "g"
	case 4:
		return "G"
	case 5:
		return "s"
	case 6:
		return "u"
	case 7:
		return "o"
	case 8:
		return "O"
	default:
		return "undefined"
	}
}

// DecodeRecordBatch decodes a version-1 RecordBatch columns blob (as packed
// by the server's BatchAcc::to_blob and documented in protos/record.proto)
// into a per-tick list of *SessionStepResponse, one element per tick in the
// batch. Replaying a recording therefore yields the SAME proto types that the
// live SimulationStepSession RPC returns. All multi-byte integers are
// little-endian.
//
// Each *SessionStepResponse has Code=0, Text="", Timestamp=tick_start+index,
// VehicleData built from the per-tick vehicle rows (VehicleType is the raw
// agent-type code converted to AgentType; the codes align exactly; Bearing is
// the on-wire centidegrees divided by 100; TravelTime is -1 because it is not
// recorded), and TlsData built by grouping the tick's traffic-light signals by
// tl_id into one TLSState per tl_id (each holding a TLGroup per group_id).
//
// It returns an error if the version byte is not 1 or if the input is
// truncated; it never panics. Empty batches (no ticks, no vehicle rows and/or
// no traffic-light groups) decode without error.
func DecodeRecordBatch(columns []byte) ([]*SessionStepResponse, error) {
	o := 0

	// need reports whether n more bytes are available from the current offset.
	need := func(n int) error {
		if o+n > len(columns) {
			return fmt.Errorf("microtraffic: truncated record blob: need %d bytes at offset %d, have %d", n, o, len(columns))
		}
		return nil
	}
	rdU32 := func() (uint32, error) {
		if err := need(4); err != nil {
			return 0, err
		}
		v := binary.LittleEndian.Uint32(columns[o : o+4])
		o += 4
		return v, nil
	}
	rdU16 := func() (uint16, error) {
		if err := need(2); err != nil {
			return 0, err
		}
		v := binary.LittleEndian.Uint16(columns[o : o+2])
		o += 2
		return v, nil
	}

	if err := need(1); err != nil {
		return nil, err
	}
	version := columns[o]
	o++
	if version != 1 {
		return nil, fmt.Errorf("microtraffic: unsupported record blob version %d (expected 1)", version)
	}

	tickStart, err := rdU32()
	if err != nil {
		return nil, err
	}
	tickCount32, err := rdU32()
	if err != nil {
		return nil, err
	}
	r32, err := rdU32()
	if err != nil {
		return nil, err
	}
	tickCount := int(tickCount32)
	r := int(r32)

	rowsPerTick := make([]int, tickCount)
	for i := range rowsPerTick {
		v, err := rdU32()
		if err != nil {
			return nil, err
		}
		rowsPerTick[i] = int(v)
	}

	total := 0
	for _, n := range rowsPerTick {
		total += n
	}
	if total != r {
		return nil, fmt.Errorf("microtraffic: rows_per_tick sum %d != total_rows %d", total, r)
	}

	vehID := make([]uint32, r)
	for i := range vehID {
		v, err := rdU32()
		if err != nil {
			return nil, err
		}
		vehID[i] = v
	}
	cell := make([]uint32, r)
	for i := range cell {
		v, err := rdU32()
		if err != nil {
			return nil, err
		}
		cell[i] = v
	}
	if err := need(r); err != nil {
		return nil, err
	}
	agentType := columns[o : o+r]
	o += r
	angle := make([]uint16, r)
	for i := range angle {
		v, err := rdU16()
		if err != nil {
			return nil, err
		}
		angle[i] = v
	}
	speed := make([]int16, r)
	for i := range speed {
		v, err := rdU16()
		if err != nil {
			return nil, err
		}
		speed[i] = int16(v)
	}
	trip := make([]uint32, r)
	for i := range trip {
		v, err := rdU32()
		if err != nil {
			return nil, err
		}
		trip[i] = v
	}

	icOff := make([]int, r)
	for i := range icOff {
		v, err := rdU32()
		if err != nil {
			return nil, err
		}
		icOff[i] = int(v)
	}
	icTotal := 0
	if r > 0 {
		icTotal = icOff[r-1]
	}
	icVals := make([]uint32, icTotal)
	for i := range icVals {
		v, err := rdU32()
		if err != nil {
			return nil, err
		}
		icVals[i] = v
	}

	tailOff := make([]int, r)
	for i := range tailOff {
		v, err := rdU32()
		if err != nil {
			return nil, err
		}
		tailOff[i] = int(v)
	}
	tailTotal := 0
	if r > 0 {
		tailTotal = tailOff[r-1]
	}
	tailVals := make([]uint32, tailTotal)
	for i := range tailVals {
		v, err := rdU32()
		if err != nil {
			return nil, err
		}
		tailVals[i] = v
	}

	gCount32, err := rdU32()
	if err != nil {
		return nil, err
	}
	gCount := int(gCount32)
	tlKeys := make([][2]uint32, gCount)
	for i := range tlKeys {
		tlID, err := rdU32()
		if err != nil {
			return nil, err
		}
		groupID, err := rdU32()
		if err != nil {
			return nil, err
		}
		tlKeys[i] = [2]uint32{tlID, groupID}
	}
	if err := need(tickCount * gCount); err != nil {
		return nil, err
	}
	tlSignals := columns[o : o+tickCount*gCount]
	o += tickCount * gCount

	// slice returns the variable-length int64 values for the given row out of
	// a cumulative-END-offset array (row i covers off[i-1]..off[i], off[-1]:=0).
	slice := func(off []int, vals []uint32, row int) []int64 {
		start := 0
		if row > 0 {
			start = off[row-1]
		}
		end := off[row]
		out := make([]int64, end-start)
		for i := range out {
			out[i] = int64(vals[start+i])
		}
		return out
	}

	responses := make([]*SessionStepResponse, 0, tickCount)

	row := 0
	for t := 0; t < tickCount; t++ {
		resp := &SessionStepResponse{
			Code:      0,
			Text:      "",
			Timestamp: int64(tickStart) + int64(t),
		}

		for k := 0; k < rowsPerTick[t]; k++ {
			resp.VehicleData = append(resp.VehicleData, &VehicleState{
				VehicleId:         int64(vehID[row]),
				VehicleType:       AgentType(agentType[row]),
				Bearing:           float64(angle[row]) / 100.0,
				Speed:             int64(speed[row]),
				Cell:              int64(cell[row]),
				IntermediateCells: slice(icOff, icVals, row),
				TravelTime:        -1,
				TripId:            int64(trip[row]),
				TailCells:         slice(tailOff, tailVals, row),
			})
			row++
		}

		// Group the tick's signals by tl_id into one TLSState per tl_id. The
		// keys are sorted by (tl_id, group_id), so same-tl_id keys are
		// contiguous and a running pointer suffices.
		var curTLS *TLSState
		var curID uint32
		for gi := 0; gi < gCount; gi++ {
			tlID := tlKeys[gi][0]
			groupID := tlKeys[gi][1]
			if curTLS == nil || tlID != curID {
				curTLS = &TLSState{Id: int64(tlID)}
				curID = tlID
				resp.TlsData = append(resp.TlsData, curTLS)
			}
			curTLS.Groups = append(curTLS.Groups, &TLGroup{
				Id:     int64(groupID),
				Signal: signalChar(tlSignals[t*gCount+gi]),
			})
		}

		responses = append(responses, resp)
	}

	return responses, nil
}
