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

Die CPU der Hostmaschine muss für Virtualisierung unterstützen. Dies sollte der Fall für alle Prozessoren der letzten zehn bis fünfzehn Jahre sein.
überprufe ob Virtualisierung unterstützt wird mit diesem Befehl:

`$ lscpu | grep Virtualization`

Sollte dieser kein Ergebnis liefern ist Virtualisierung mit KVM nicht möglich.

Wenn der folgende Befehl neben 'kvm' nicht 'kvm_intel' oder 'kvm_amd' auflistet, dann muss für AMD-Prozessoren SVM/AMD-V und für Intel-Prozessoren VT-x im BIOS/UEFI aktiviert werden.

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

Anschließend kann der libvirtd deamon gestartet werden.

OpenRC:
`# rc-service libvirtd start && rc-update add libvirtd default` 

systemd:
`# systemctl enable --now libvirtd` 


### Netzwerk

#### Standard Netzwerk mit libvirt

Für gewöhnlich sollte an diesem Punkt schon ein Standard-Netzwerk konfiguriert sein und hier auftauchen:

`# virsh net-list --all` 

Wenn das Netzwerk noch nicht aktiviert ist, starte es mit diesem Befehl:

`# virsh net-start default`

Änderungen am Netzwerk können mit diesem Befehl vorgenommen werden:

`# virsh net-edit default`

Netzwerk ausschalten:

`# virsh net-destroy default`

Netzwerk nach Boot des Hostgeräts automatisch starten:

`# virsh net-autostart default` 

Um eine Verbindung mit dem Internet in der virtuellen Maschine herzustellen ist das von libvirt automatisch konfigurierte 'default'-Netzwerk ausreichend. Soll die virtuelle Maschine jedoch auch für Geräte außerhalb dieses Standard-Netzwerkes sichtbar sein, dann muss eine Netzwerkbrücke eingerichtet werden.


#### Netzwerkbrücke 

Zum Erstellen der Netzwerkbrücke können verschieden Methoden verwendet werden. Virt-manager bietet dafür ein GUI, ebenso NetworkManager, wenn für diesen ein graphisches Frontend installiert wurde. Bridge-utils und NetworkManager bieten mit bridge-ctl und nmcli die Möglichkeit dies auf der Kommandozeile zu erledigen wie auch iproute2 etc. Eine andere Lösung wird durch systemd-networkd für Systeme mit Systemd und mit netifrc für Systeme mit OpenRC geboten.

Für die folgenden Befehle sind die Interface-Namen und im Falle einer statischen IP-Konfiguration die IP-Addressen der zu überbrückenden Netzwerk-Interfaces von Interesse.

`$ ip addr` 

Es ist ebenfalls zu bedenken, dass während der folgenden Prozedur die Verbindung zum Internet verloren gehen kann.

OpenRC:

Stelle sicher, dass die Interfaces, die Teil der Brücke sein sollen, nicht in /etc/init.d/ vorhanden sind.

`# rc-update delete net.<Name des Interfaces> boot`
`# rm /etc/init.d/net.<Name des Interfaces>`

Dieser Schritt ist für alle relevanten Interfaces zu wiederholen.

Ändere oder erstelle /etc/conf.d/net:

`/etc/conf.d/net`
```
config_enp1s0="null"
bridge_br0="enp1s0"
config_br0="dhcp"

bridge_forward_delay_br0=0
bridge_hello_time_br0=1000
```

Anstatt 'enp1s0' ist der Name des zu überbrückenden Interfaces einzusetzen und an Stelle von '"null"' können alle anderen Interfaces, die Teil der Brücke sein sollen, eingesetzt werden.
'br0' ist der Name der Netzwerkbrücke und kann nach Belieben geändert werden.
Bei einer statischen IP ist diese anstelle von 'dhcp' anzugeben.
Die letzten zwei Zeilen sind notwendig, um Paketverlust unmittelbar nach starten der Netzwerkbrücke zu vermeiden.

Anschließend muss die Netzwerkbrücke gestartet werden:

`# ln -s /etc/init.d/net.lo /etc/init.d/net.br0`
`# rc-service net.br0 start && rc-update add net.br0 default`


Systemd:

Es sind folgende Dateien zu erstellen:

`/etc/systemd/network/Bridge0.netdev`
```
[NetDev]
Name=br0
Kind=bridge

```

`/etc/systemd/network/Ethernet0.network`
```
[Match]
Name=enp1s0 # Und alle anderen zu überbrückenden Interfaces

[Network]
Bridge=br0

```

`/etc/systemd/network/Bridge0.network`
```
[Match]
Name=br0

[Network]
DHCP=ipv4

``` 


Oder im Falle einer statischen IP-Konfiguration:

`/etc/systemd/network/Bridge0.network`
```
[Ma[Match]
Name=br0

[Network]
DNS=192.168.1.1
Address=192.168.1.2/24
Gateway=192.168.1.1tch]
Name=br0

[Network]
DNS=192.168.1.1
Address=192.168.1.2/24
Gateway=192.168.1.1

```

Um die Netzwerkbrücke zu starten muss der 'systemd-networkd'-Dienst gestartet werden:

`# systemctl enable --now systemd-networkd` 



## Einrichtung virtuelle Maschine

Vor der Einrichtung der virtuellen Maschine kann bereits eine img-Datei mit der gewünschten Größe, dem Format und am bevorzugtem Ort erstellt werden:

`$ qemu-img create -f qcow2 <VM-Name>.img <Größe in GB>G`

Auf dieses Image kann dann die virtuelle Mashine installiert werden:

```
$ virt-install  \
  --name <VM-Name> \
  --memory <Arbeitsspeicher in MB> \
  --vcpus=<Anzahl an CPU-Threads> \
  --disk <Pfad zur oben erstellten img-Datei> \
  --os-variant=<Name und Version der Linuxdistribution oder win7/8/10/etc> \
  --cpu host \  
  --cdrom <Pfad zum iso-Abbildd des zu installierenden OSes> \
  --network default \
  --graphics spice
  --virt-type kvm

```
Im Fall, das vorher keine img-Datei erstellt wurde, kann die disk-Option des eben aufgeführten Befehls auch geändert werden in:

`--disk size=<Größe in GB>,format=qcow2`

Um die passende Bezeichnung für die '--os-variant'-Option herauszufinden kann dieser Befehl verwendet werden:

`$ virt-install --osinfo list`

Wenn für die virtuelle Maschine nicht das von libvirt erstellte default Netzwerk verwendet werden soll, kann die '--network'-Option auch geändert werden, um z.B. die vorher konfigurierte Netzwerkbrücke zu verwenden:

`--network bridge=br0`

Anschließend wird automatisch in die Live-Umgebung des iso-Abbilds gebootet, wo der Standardinstallationsprozess durchgeführt werden kann.
 

## Verwaltung und Nutzung der virtuellen Maschine

Starten der virtuellen Maschine:

`$ virsh start <VM-Name>`


Automatisches Starten:

`$ virsh autostart <VM-Name>` 


Herunterfahren:

`$ virsh shutdown <VM-Name>`


Ausschalten:

`$ virsh destroy <VM-Name>`


Ändern der Eigenschaften der VM:

`$ virsh edit <VM-Name>`


Löschen der VM:

`$ virsh undefine <VM-Name>` 

Das Image auf dem die virtuelle Maschine installiert ist, muss separat gelöscht werden.


### SPICE

Um auf den Display-Output der VM zuzugreifen wird ein SPICE Client benötigt, wie spice-gtk oder virt-viewer.

Die Verbindungsaddresse für die VM ist herauszufinden mit:

`$ virsh domdisplay <VM-Name>`

Die Verbindung kann hergestellt werden mit:

`$ remote-viewer spice://127.0.0.1:5900`

'127.0.0.1:5900' sind die Standardaddresse und Port, wenn nichts anderes definiert wurde.




