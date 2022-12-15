#!/bin/bash
cpu=0
iteration=1
while [[ $cpu < 80 ]]; do
  `virt-clone --original qemuvm0 --name qemuvm$iteration --file ~/VM-Tests/QEMU/qemuvm$iteration.img`
  `virsh start qemuvm$iteration`
  echo $iteration
  iteration=$(( iteration + 1 ))
done
echo "cpu utilized"
