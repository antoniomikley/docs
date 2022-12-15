#!/bin/bash
while :
do
	t1=`date +%s%N`
	virsh start qemuvm0
	s=`virsh domdisplay qemuvm0`
	remote-viewer $s	
	wait
	t2=`date +%s%N`
	virsh destroy qemuvm0

	echo "scale=3 ; ($t2-$t1) / 1000000000" | bc >> /home/antonio/times
done
