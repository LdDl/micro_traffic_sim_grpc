package microtraffic

import (
	"encoding/binary"
	"fmt"
)

// RecordedVehicle is a single decoded vehicle row from a RecordBatch
// columns blob. Tick is the absolute simulation tick the row belongs to
// (computed from tick_start plus the per-tick row grouping). AgentType is
// the raw agent-type code (0=Undefined 1=Car 2=Bus 3=Taxi 4=Pedestrian
// 5=Truck 6=LargeBus); display name mapping is left to the caller. Angle
// is the bearing in degrees (the on-wire centidegrees divided by 100).
type RecordedVehicle struct {
	Tick              uint64
	VehicleID         uint32
	Cell              int64
	AgentType         uint8
	Angle             float64
	Speed             int16
	TripID            uint32
	IntermediateCells []uint32
	TailCells         []uint32
}

// RecordedSignal is a single decoded traffic-light signal sample for one
// (TLID, GroupID) key at an absolute simulation Tick. Signal is the signal
// character (e.g. "r", "y", "g", "G", "s", "u", "o", "O" or "undefined").
type RecordedSignal struct {
	Tick    uint64
	TLID    uint32
	GroupID uint32
	Signal  string
}

// DecodedBatch is the structured result of decoding a RecordBatch columns
// blob: the vehicle rows (in tick order, then in the on-wire order within
// each tick) and the traffic-light signal samples (in tick order, then in
// the on-wire key order within each tick).
type DecodedBatch struct {
	Vehicles []RecordedVehicle
	Signals  []RecordedSignal
}

// signalName maps the on-wire signal code to its string representation.
func signalName(c byte) string {
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
// into a DecodedBatch. All multi-byte integers are little-endian.
//
// It returns an error if the version byte is not 1 or if the input is
// truncated. Empty batches (no vehicle rows and/or no traffic-light groups)
// decode without error into empty slices.
func DecodeRecordBatch(columns []byte) (*DecodedBatch, error) {
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

	// slice returns the variable-length values for the given row out of a
	// cumulative-END-offset array (row i covers off[i-1]..off[i], off[-1]:=0).
	slice := func(off []int, vals []uint32, row int) []uint32 {
		start := 0
		if row > 0 {
			start = off[row-1]
		}
		end := off[row]
		out := make([]uint32, end-start)
		copy(out, vals[start:end])
		return out
	}

	batch := &DecodedBatch{}

	row := 0
	for t := 0; t < tickCount; t++ {
		tick := uint64(tickStart) + uint64(t)
		for k := 0; k < rowsPerTick[t]; k++ {
			batch.Vehicles = append(batch.Vehicles, RecordedVehicle{
				Tick:              tick,
				VehicleID:         vehID[row],
				Cell:              int64(cell[row]),
				AgentType:         agentType[row],
				Angle:             float64(angle[row]) / 100.0,
				Speed:             speed[row],
				TripID:            trip[row],
				IntermediateCells: slice(icOff, icVals, row),
				TailCells:         slice(tailOff, tailVals, row),
			})
			row++
		}
	}

	for t := 0; t < tickCount; t++ {
		tick := uint64(tickStart) + uint64(t)
		for gi := 0; gi < gCount; gi++ {
			batch.Signals = append(batch.Signals, RecordedSignal{
				Tick:    tick,
				TLID:    tlKeys[gi][0],
				GroupID: tlKeys[gi][1],
				Signal:  signalName(tlSignals[t*gCount+gi]),
			})
		}
	}

	return batch, nil
}
