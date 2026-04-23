#!/usr/bin/bash
# for testing speed-from-fan cli arg

for ((i = 0; i <= 100; i=$i+5)); do
	echo $i
	sudo framework_tool --fansetduty $i
	sleep 1
done
sleep 5

for ((i = 0; i <= 100; i=$i+5)); do
	echo $((100-$i))
	sudo framework_tool --fansetduty $((100-$i))
	sleep 1
done
