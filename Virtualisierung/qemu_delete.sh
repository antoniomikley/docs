i=1
while [[ 1 == 1 ]]; do
  virsh destroy qemuvm$i
  virsh undefine qemuvm$i
  i=$(( i + 1 ))
  
done
