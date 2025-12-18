set datafile separator ";"
set terminal gif animate delay 100 size 800,600
set output "examples/rust_client/output.gif"

# Find step range from vehicle data - match ONLY "step;vehicle_id" header
stats '< sed -n "/^step;vehicle_id/,/^tl_step;/p" examples/rust_client/output.txt | tail -n +2 | head -n -1' using 1 nooutput
first_step = STATS_min
last_step = STATS_max

# Find x/y range from grid cells only
stats '< sed -n "/^cell_id;x;y/,/^step;vehicle_id/p" examples/rust_client/output.txt | tail -n +2 | head -n -1' using 2 nooutput
min_x = STATS_min
max_x = STATS_max
stats '< sed -n "/^cell_id;x;y/,/^step;vehicle_id/p" examples/rust_client/output.txt | tail -n +2 | head -n -1' using 3 nooutput
min_y = STATS_min
max_y = STATS_max

set xrange [min_x-0.5:max_x+0.5]
set yrange [min_y-0.5:max_y+0.5]
set xtics 1
set ytics 1
set grid
set key outside

set palette defined ( 0 "red", 1 "blue", 2 "green", 3 "orange", 4 "purple", 5 "cyan", 6 "magenta", 7 "brown")
set cbrange [0:9]
unset colorbox

do for [i=first_step:last_step] {
    set title sprintf("Step: %d", i)
    plot \
        '< grep ";cell;coordination" examples/rust_client/output.txt' using 2:3:(0.25) with circles lc rgb "#D3D3D3" lw 1.5 title "Coordination zones", \
        '< grep ";cell;birth" examples/rust_client/output.txt' using 2:3 with points pt 7 ps 4 lc rgb "#B3FFB3" title "Spawn zones", \
        '< grep ";cell;death" examples/rust_client/output.txt' using 2:3 with points pt 7 ps 4 lc rgb "#FFB3FF" title "Despawn zones", \
        '< grep ";cell;common" examples/rust_client/output.txt' using 2:3 with points pt 7 ps 3 lc rgb "#666666" title "Common zones", \
        '< grep "forward" examples/rust_client/output.txt | grep -v "tl_id"' using 2:3:($4-$2):($5-$3) with vectors head filled size screen 0.03,15,45 lc rgb "blue" lw 2 title "Forward", \
        '< grep "left" examples/rust_client/output.txt' using 2:3:($4-$2):($5-$3) with vectors head filled size screen 0.03,15,45 lc rgb "green" lw 2 title "Left maneuver", \
        '< grep "right" examples/rust_client/output.txt' using 2:3:($4-$2):($5-$3) with vectors head filled size screen 0.03,15,45 lc rgb "red" lw 2 title "Right maneuver", \
        '< sed -n "/^tl_id;x;y/,/^tl_id;controlled_cell/p" examples/rust_client/output.txt | tail -n +2 | head -n -1' using 2:3 with points pt 13 ps 3 lc rgb "0x7b1085" title "Traffic Light", \
        '< sed -n "/^tl_id;controlled_cell/,/^cell_id/p" examples/rust_client/output.txt | tail -n +2 | head -n -1' using 3:4:(0.4) with circles dashtype 2 lw 2 lc rgb "#2bdfd0" title "TL Control Zone", \
        sprintf('< sed -n "/^tl_step;tl_id;group_id;cell_id;x;y;signal/,\$p" examples/rust_client/output.txt | tail -n +2 | grep "^%d;.*g$"', i) using 5:6:(0.5) with circles dashtype 2 lw 3 lc rgb "0x00FF00" title "Signal group (GREEN)", \
        sprintf('< sed -n "/^tl_step;tl_id;group_id;cell_id;x;y;signal/,\$p" examples/rust_client/output.txt | tail -n +2 | grep "^%d;.*r$"', i) using 5:6:(0.5) with circles dashtype 2 lw 3 lc rgb "0xFF0000" title "Signal group (RED)", \
        '< sed -n "/^step;vehicle_id/,/^tl_step;/p" examples/rust_client/output.txt | tail -n +2 | head -n -1' using ($1==i ? $8 : 1/0):($1==i ? $9 : 1/0):(int($2) % 10) with points pt 7 ps 5 lc palette notitle, \
        '< sed -n "/^step;vehicle_id/,/^tl_step;/p" examples/rust_client/output.txt | tail -n +2 | head -n -1' using ($1==i ? $8 : 1/0):($1==i ? $9 : 1/0):(sprintf("id=%d", int($2))) with labels offset 0,0.5 font ",8" notitle
}
set output
