# HTTPD Webserver mit Owncloud

## Benötigte Software

### httpd

Lade herunter und installiere httpd/apache2
Starte httpd Dienst

### mariadb

Lade herunter und istalliere mariadb
Starte mariadb/mysqld Dienst

`# mysql_secure_installation`

### php

Lade herunter und installiere php, php-common, php-mysqlnd, php-xml, php-json, php-gd, php-mbstring, php-zip, php-posix und vlt. noch mehr

Muss php7.4 sein.

### owncloud

`$ cd /var/www/owncloud/`

`# wget https://download.owncloud.org/server/stable/owncloud-latest-stable.tar.bz2`

`# tar xjf owncloud-latest-stable.tar.bz2`

`# chown -R apache.apache owncloud`

`# chmod -R 755 owncloud` 

`# rm -f owncloud-latest-stable.tar.bz2` 

## Einrichtung

### MySQL Datenbank und Nutzer erstellen

`mysql -u root -p`
```
> CREATE USER IF NOT EXISTS 'admin'@'hostname' IDENTIFIED BY 'password';
> GRANT ALL PRIVILEGES ON *.* TO 'admin'@'hostname' WITH GRANT OPTION;
> FLUSH PRIVILEGES;
> quit
```

### SSL Zertifikate erstellen

`# openssl genrsa -out myhost.com.key 2048`

`# openssl req -new -key hostname.com.key -out hostname.com.csr -sha512` 

`# openssl x509 -req -days 365 -in hostname.com.csr -signkey hostname.com.key -out hostname.com.crt -sha512` 

`# cp hostname.com.crt /etc/pki/tls/certs/` 

`# cp hostname.com.key /etc/pki/tls/private/hostname.com.ke`

`# cp hostname.com.csr /etc/pki/tls/private/hostname.com.csr`

`# restorecon -RvF /etc/pki` 


### Zertifikate installieren

`# sudo mv key_file.key /etc/pki/tls/private/hostname.com.key`

`# sudo mv certificate.crt /etc/pki/tls/certs/hostname.com.crt` 

`# restorecon /etc/pki/tls/private/myhost.com.key` 

`# restorecon /etc/pki/tls/certs/myhost.com.crt`

`# chown root.root /etc/pki/tls/private/myhost.com.key` 

`# chown root.root /etc/pki/tls/certs/myhost.com.crt`

`# chmod 0600 /etc/pki/tls/private/myhost.com.key`

`# chmod 0600 /etc/pki/tls/certs/myhost.com.crt` 

`/etc/httpd/conf.d/ssl.conf`
```
<VirtualHost _default_:443>
...
SSLCertificateFile /etc/pki/tls/certs/hostname.com.crt
...
SSLCertificateKeyFile /etc/pki/tls/private/hostname.com.key
...
</VirtualHost>
```

### Installation Owncloud

http://hostname/owncloud/

### Erweiterter Zugang und Firewall

`/etc/httpd/conf.downcloud.conf`
```
<Directory /var/www/owncloud>
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

`# firewall-cmd --permanent --add-service=http`

`# sudo firewall-cmd --permanent --add-service=https`

### Trusted Domains

`/var/www/owncloud/config/config.php`
```
‘trusted_domains’ =>
array (
0 => ‘localhost’,
1 => ‘127.0.0.1’,
2 => '192.168.1.*',
3 => 'hostname',
```

HTTPD Dienst neu starten oder reboot VM

