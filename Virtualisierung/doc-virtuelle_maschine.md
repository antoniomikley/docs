# Virtualisierung

Dieses Dokument beschreibt das Erstellen einer virtuellen Maschine mittels KVM, QEMU und libvirt in Linux.

## Voraussetzungen
### Benötigte Software

- libvirt
- qemu
- virt-install
- virt-viewer

Es ist ebenfalls sicher zu stellen, dass folgende Pakete vorhanden sind, sollten diese nicht bereits als Abhängigkeiten der oben aufgeführten Programme installiert worden sein.

- iptables
- dnsmasq
- iproute2
- virsh (normalerweise Teil von libvirt)
- bridge-utils (optional)

### Hardwarevoraussetzungen

Die CPU der Hostmaschine muss für KVM Virtualisierung unterstützen. Dies sollte in der Fall für alle Prozessoren der letzten zehn bis fünfzehn Jahre sein.
überprufe ob Virtualisierung unterstützt wird mit diesem Befehl:

`$ lscpu | grep Virtualization`

Sollte dieser kein Ergebnis liefern ist Virtualisierung mit KVM nicht möglich.

Wenn der folgende Befehl nebrn 'kvm' nicht 'kvm_intel' oder 'kvm_amd' auflistet, dann muss für AMD-Prozessoren SVM/AMD-V und für Intel-Prozessoren VT-x im BIOS/UEFI aktiviert werden.

`$ lsmod | grep kvm`

Sind diese Optionen bereits aktiviert, dann fehlen die notwendigen Kernelmodule.

### Kernel

Für Virtualisierung müssen bestimmte Module im Kernel enthalten sein.
Dies sollte für die meisten Distributionen der Fall sein, außer der Kernel wurde vom Nutzer selbst konfiguriert und kompiliert.
In diesem Fall ist das Gentoo-Wiki sehr hilfreich.

- https://wiki.gentoo.org/wiki/Libvirt#Kernel
- https://wiki.gentoo.org/wiki/QEMU#Kernel

## Konfiguration

Bevor virtuelle Maschinen eingerichtet werden können, muss noch der libvirtd daemon konfiguriert werden.

### Authorisierung

Für vollständige Funktionalität benötigt der Nutzer ausreichende Authorisierung und sollte deshalb zur 'livirt'-Gruppe hinzugegügt werden.

`# usermod -a -G libvirt <Nutzer>` 

Ebenso sollten folgende Zeilen in der Konfigurationsdatei enthalten sein:

`/etc/libvirt/libvirtd.conf`
```
auth_unix_ro = "none"
auth_unix_rw = "none"
unix_sock_group = "libvirt"
unix_sock_ro_perms = "0777"
unix_sock_rw_perms = "0770"

```
### Dienst

Anschließend kann der libvirtd deamon gestarted werden.

OpenRC:
`# rc-service libvirtd start && rc-update add libvirtd default` 

systemd:
`# systemctl enable --now libvirtd` 

### Netzwerk

Für gewöhnlich sollte an diesem Punkt schon ein Standard-Netzwerk konfiguriert sein und hier auftauchen:

`# virsh net-list --all` 

Dieses Netzwerk sollte erstmal ausreichen.

## Einrichtung virtuelle Maschine

Vor der Einrichtung der virtuellen Maschine kann bereits eine img-Datei mit der gewünschten Größe, dem Format und am bevorzugtem Ort erstellt werden:

`$ qemu-img create -f qcow2 <VM-Name>.img <Größe in GB>G`

Auf dieses Image kann dann die virtuelle Mashine installiert werden:

```
$ virt-install  \
  --name <VM-Name> \
  --memory <Arbeitsspeicher in MB> \
  --vcpus=<Anzahl an CPU-Threads> \
  --disk <Pfad zur oben erstellten img-Datei>
  --cpu host                \
  --cdrom <Pfad zum iso-Abbildd des zu installierenden OSes> \
  --network default \
  --virt-type kvm

```
Im Fall das vorher keine img-Datei vorher erstellt wurde, kann die disk-Option des eben aufgeführten Befehls auch geändert werden in:

`--disk size=<Größe in GB>,format=qcow2`

Für bessere Performance können auch noch folgende Option hinzugefügt werden:

`--os-variant=<Name und Version der Linuxdistribution oder win7/8/10/etc>`


