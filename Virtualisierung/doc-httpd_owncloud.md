# HTTPD Webserver mit ownCloud

Dieses Dokument beschreibt die Einrichtung einens Apache-Webservers mit ownCloud.

1. [Vorraussetzungen](#Vorraussetzungen)
   - [Benötigte Software](#Benötigte-Software) 
   - [Netzwerk](#Netzwerk) 
2. [Installation und Konfiguration](#Installation-und-Konfiguration)
   - [Apache](#Apache)
     - [SSL-Zertifikate](#SSL-Zertifikate)
   - [MariaDB](#MariaDB)
   - [ownCloud](#ownCloud)
     - [Installation](#Installation)
     - [Berechtigungen](#Berechtigungen)
     - [Installation abschließen](#Installation-abschließen)
     - [Firewall und Trusted Domains](#Firewall-und-Trusted-Domains) 


## Vorraussetzungen

### Benötigte Software

- httpd/apache2
- mariadb
- php
- owncloud
- openssl

PHP muss Version 7.4 sein, da ownCloud neuere Versionen nicht unterstützt, und wird daher wahrscheinlich nicht in den offiziellen Repositories der meisten Linux-Distributionen sein. Owncloud muss von deren Webseite bezogen werden. OpenSSL ist nur notwendig für die generierung von SSL-Zertifikaten und ist daher optional.

Weiterhin wird eine Reihe an PHP-Modulen benötigt:
php-common, php-mysqlnd, php-xml, php-json, php-gd, php-mbstring, php-zip, php-posix und vielleicht noch mehr.

### Netzwerk

Dieser Schritt ist nur notwendig, wenn der Webserver in einer virtuellen Maschine installiert wird. 

Verfügt die Hostmaschine über eine Etherneverbindung kann der Ethernetadapter gebridged werden, um den Webserver in der VM für andere Geräte zugänglich zu machen. Eine Anleitung dazu ist in [der Dokumenation zu virtuellen Maschinen](doc-virtuelle_maschine.md) zu finden. Ist der Host über Wifi mit dem Internet verbunden, dann ist es zu empfehlen, dies zu ändern.

Angenommen die virtuelle Maschine hat eine Verbindung zum Internet über das von libvirt bereitgestellte 'default'-Netzwerk ist diese Datei mit folgendem Inhalt zu erstellen, während die VM heruntergefahren ist.

`/etc/libvirt/hooks/qemu`
```bash
#!/bin/bash

# Hier Name der VM einfügen
if [ "${1}" = "VM-Name"]; then
  GUEST_IP=192.168.122.20 # Hier IP der VM
  GUEST_GW=192.168.122.0/24 # Wie GUEST_IP, aber letzte Zahl ist 0 und mit /24
  WIFI=wlp3s0 # Bezeichnung für Wifiadapter 
  HOST_PORT=9001 # Hier der Port auf dem Host, welcher weitergeleitet wird, vorzugsweise einer zwischen 1024 - 65535
  
  if [ "${2}" = "stopped" ] || [ "${2}" = "reconnect" ]; then
	  /sbin/iptables -D FORWARD -o virbr0 -p tcp -d $GUEST_IP --dport 443 -j ACCEPT
	  /sbin/iptables -t nat -D PREROUTING -p tcp --dport $HOST_PORT -j DNAT --to $GUEST_IP:443
  fi

  if [ "${2}" = "start" ] || [ "${2}" = "reconnect" ]; then
	  /sbin/iptables -I FORWARD -o virbr0 -p tcp -d $GUEST_IP --dport 443 -j ACCEPT
	  /sbin/iptables -t nat -I PREROUTING -p tcp --dport $HOST_PORT -j DNAT --to $GUEST_IP:443
	  /sbin/iptables -A PREROUTING -d $HOST_IP -p tcp -m tcp --dport $HOST_PORT -j DNAT --to-destination $GUEST_IP:443
	  /sbin/iptables -t filter -A FORWARD -d $GUEST_IP/24 -o virbr0 -p tcp -m tcp --syn -m conntrack --ctstate NEW -m multiport --dports $HOST_PORT -j ACCEPT
	  /sbin/iptables -A FORWARD -d $GUEST_GW -o virbr0 -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT
	  /sbin/iptables -A FORWARD -s $GUEST_GW virbr0 -j ACCEPT
	  /sbin/iptables -t nat -A POSTROUTING -s $GUEST_GW ! -d $GUEST_GW -p tcp -j MASQUERADE --to-ports 1024-6553
	  /sbin/iptables -t nat -A POSTROUTING -s $GUEST_GW ! -d $GUEST_GW -p udp -j MASQUERADE --to-ports 1024-65535
	  /sbin/iptables -t nat -A POSTROUTING -s $GUEST_GW ! -d $GUEST_GW -j MASQUERADE
	  /sbin/iptables -t filter -A FORWARD -d $GUEST_IP/24 -o virbr0 -p tcp -m tcp --syn -m conntrack --ctstate NEW -m multiport --dports 80,443 -j ACCEPT
	  /sbin/iptables -t filter -A INPUT -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT
	  /sbin/iptables -t filter -A INPUT -i lo -j ACCEPT
	  /sbin/iptables -t filter -A INPUT -p icmp --icmp-type 8 -m conntrack --ctstate NEW -j ACCEPT
	  /sbin/iptables -t filter -A FORWARD -i $WIFI -o virbr0 -j ACCEPT
  fi
fi  
```
Diese Hook wird immer ausgeführt, wenn eine VM gestartet wird.
Sollte der Webserver danach immer noch nicht erreichbar sein, dann starte den libvirtd-Dienst und die VM neu.

## Installation und Konfiguration

### Apache

Die Standard-Konfiguration ist ausreichend für diesen Anwendungsfall, sollten keine SSL-Zertifikate erwünscht sein und es muss nur der httpd-Dienst gestartet werden.

In manchen Fällen muss jedoch noch vorher eingerichtet werden, dass die Webapplikation von anderen Geräten errreicht werden kann.
Erstelle oder modifiziere dafür diese Datei mit folgendem Inhalt:

`/etc/httpd/conf.d/owncloud.conf`
```
<Directory /usr/share/webapp>
    <IfModule mod_authz_core.c>
        # Apache 2.4
        Require all granted
    </IfModule>
    <IfModule !mod_authz_core.c>
        # Apache 2.2
        Order Deny,Allow
        Allow from all
    </IfModule>
</Directory>
```

#### SSL-Zertifikate

Es ist möglich selbstsignierte SSL-Zertifikate mit OpenSSL zu erstellen. Eine Alternative für nicht selbstsignierte Zertifikate wird mit Let'sEncrypt geboten.

Erstellung der Zertifikate mit OpenSSL:

`# openssl genrsa -out Host-Name.key 2048`

`# openssl req -new -key Host-Name.key -out Host-Name.csr -sha512`

`# openssl x509 -req -days 365 -in Host-Name.csr -signkey Host-Name.key -out Host-Name.crt -sha512`

Die Zertifikate müssen anschließend an dem richtigen Ort abgelegt werden:

`# mv Host-Name.key /etc/pki/tls/private/Host-Name.key`

`# mv Host-Name.crt /etc/pki/tls/certs/Host-Name.key`

Für Linux-Distributionen, die aktiv Gebrauch von SELinux-Funktionalität machen (Fedora, RedHat, openSUSE,...):

`# restorecon /etc/pki/tls/private/Host-Name.key` 

`# restorecon /etc/pki/tls/certs/Host-Name.crt`

Falls dies nicht schon der Fall ist, müssen root die Rechte an diesen Dateien übertragen werden:

`# chown root.root /etc/pki/tls/private/Host-Name.key` 

`# chown root.root /etc/pki/tls/certs/Host-Name.crt`
 
`# chmod 0600 /etc/pki/tls/private/Host-Name.key` 

`# chmod 0600 /etc/pki/tls/certs/Host-Name.crt`

Um diesen Prozess abzuschließen, muss der Pfad zu den .key und .crt Dateien beim Webserver hinterlegt werden. Dazu müssen diese Zeilen in dieser Konfigurationsdatei einkommentiert und geändert werden:

`/etc/httpd/conf.d/ssl.conf`
```
<VirtualHost _default_:443>
...
SSLCertificateFile /etc/pki/tls/certs/Host-Name.crt
...
SSLCertificationKeyFile /etc/pki/private/Host-Name.key
...
</VirtualHost>
```
### MariaDB

Nachdem das mariadb-server Paket heruntergeladen und installiert wurde, muss der mariadb-Dienst gestartet werden und sollte auch automatisch bei Boot gestartet werden.
Um die Installation sicher zu stellen und einen Nutzer anzulegen:

`# mysql_secure_installation`

`# mysql -u root -p`
```
> CREATE USER IF NOT EXISTS 'admin'@'Host-Name' IDENTIFIED BY 'Passwort';
> GREATE DATABASE IF NOT EXISTS owncloud;
> GRANT ALL PRIVILEGES ON *.* TO 'admin'@'Host-Name' WITH GRANT OPTION;
> FLUSH PRIVILEGES;
> quit
```

### ownCloud

Letztendlich muss noch die Webapplikation installiert und eingerichtet werden.

#### Installation

Das Archiv muss dafür nur in das korrekte Verzeichnis entpackt werden:

`$ cd /var/www/html/`

`# wget https://download.owncloud.com/server/stable/owncloud-complete-latest.tar.bz2`

`# tar xjf owncloud-complete-latest.tar.bz2`

#### Berechtigungen

Außerdem benötigt der Webserver die Zugriffsrechte für das eben entpackte Verzeichnis und dessen Inhalte:
 
`# chown -R apache.apache owncloud`

`# chmod -R 755 owncloud`

Anstatt apache kann der Inhaber je nach Linux-Distribution einen anderen Namen haben (z.B. www-data für Ubuntu).

Für SELinux sind hier noch zusätzliche Schritte nötig:

`# semanage fcontext -a -t httpd_sys_rw_content_t '/var/www/html/owncloud/data(/.*)?'`

`# semanage fcontext -a -t httpd_sys_rw_content_t '/var/www/html/owncloud/config(/.*)?'`

`# semanage fcontext -a -t httpd_sys_rw_content_t '/var/www/html/owncloud/apps(/.*)?'`

`# semanage fcontext -a -t httpd_sys_rw_content_t '/var/www/html/owncloud/apps-external(/.*)?'`

`# semanage fcontext -a -t httpd_sys_rw_content_t '/var/www/html/owncloud/.htaccess'`

`# semanage fcontext -a -t httpd_sys_rw_content_t '/var/www/html/owncloud/.user.ini'`

`# restorecon -Rv '/var/www/html/owncloud/'`

#### Installation abschließen

Starte den httpd-Dienst neu und besuche anschließend diese Seite im Browser, um die Installation abzuschließen:

> http://localhost/owncloud/

Alternativ kann die Installation in der Befehlszeile abgeschlossen werden:

`$ cd /var/www/html/owncloud/`

```
sudo -u apache ./occ maintenance:install \
   --database "mysql" \
   --database-name "owncloud" \
   --database-user "root"\
   --database-pass "Passwort" \
   --admin-user "admin" \
   --admin-pass "Passwort"
```

Anstatt apache muss hier wieder der entsprechende Nutzer-/Gruppenname verwendet werden.
#### Firewall und Trusted Domains

Damit ownCloud auch von anderen Geräten erreichbar ist, sollten die richtigen Firewallregeln gesetzt werden:

`# firewall-cmd --permanent --add-service=http`

`# firewall-cmd --permanent --add-service=https`

`# firewall-cmd --reload` 

Um erreichbar über andere Domainnamen zu sein, außer localhost, sind diese zu den Trusted Domains hinzuzufügen:

`/var/www/owncloud/config/config.php`
```
‘trusted_domains’ =>
array (
0 => ‘localhost’,
1 => ‘127.0.0.1’,
2 => 'HOST_IP',
3 => 'Host-Name',
```
Abschließend muss noch der httpd-Dienst neu gestartet werden.

