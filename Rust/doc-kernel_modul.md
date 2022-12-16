
# Stringsuche im Kernel mit Rust

Dieses Dokument beschreibt die Entwicklung eines Linux-Kernelmoduls in Rust zur Suche von Schlüsselwörtern 
und einer komlementären Client-Anwendung, welche es möglichst effiziene und einfach machen soll, mit 
dem Kernelmodul zu interagieren.

---
Toc
---

## Vorraussetzungen
### Rust

Die meisten Linux-Distributionen stellen ein einfaches Rust Packet in ihren Repositories zur Verfügung, 
welches jedoch in diesem Fall nicht ausreicht. Stattdessen kann Rust mit den notendigen Entwicklungswerkzeugen 
mittels Rusts eigenen Installer rustup installiert werden. Sollte dieser nicht in den Repositories vorhanden sein, 
ist es möglich, diesen mittels Curl zu installieren:

`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

Zu beachten ist dabei, dass eine spezifische Version der Rust-Toolchain installiert werden muss, da für Rust im Kernel
unstable Features verwendet werden, welche in älteren Versionen vorhanden sind oder in in neueren Verionen geändert werden können.
Dies ist nur relevant fÜr die Entwicklung des Kernelmoduls selbst und nicht für die Client-Anwendung.

Genauere Anleitungen sind [hier](https://github.com/Rust-for-Linux/linux/blob/rust/Documentation/rust/quick-start.rst) zu finden.

### Kernel

Rust wird ab Version 6.1 im Kernel unterstützt, jedoch nur mit minimalen Features. Für die Funktionalität des Kernelmoduls 
muss stattdessen eine modifizierte Version des Kernels des Rust-for-Linux-Projekts kompiliert werden.

`https://github.com/Rust-for-Linux/linux`

Außerdem mus der Kernel mit libclang kompiliert werden, was Teil von LLVM ist.



## Stringsuche

Das Durchsuchen einer Textdatei nach einem Schlüsselwort ist möglich in dem auf der Befehlszeile der Client-Anwendung 
der Pfad zu der zu durchsuchenden Datei sowie das Schlüsselwort geliefert wird.

`# /Pfad-zur-Client-App /Pfad-zur-Textdatei 'Schlüsselwort'`

Die Client-Anwendung öffnet dann die zu durchsuchende Datei sowie das Character-Device unter `/dev/test0` und versucht 
dann zwei Tcp-Streams zum Charecter-Device aufzubauen. Über einen Stream wird das Schlüsselwort zum Kernelmodul geschickt 
und auf dem anderen Stream werden die Ergebnisse der Suche zurückgeschickt.
Anschließend liest die Client-Anwendung eine bestimmte Anzahl an Bytes aus der Textdatei und schreibt diese in das Charecter-Device.
Dort werden die Bytes überprüft, ob sie mit den Bytes des Schlüsselwortes übereinstimmen. Die Postionen an welchen übereinstimmungen gefunden wurden, werden dann über den zweiten Tcp-Stream an die Client-Anwendung zurückgeschickt, wo sie in einem separaten Thread gezählt und in eine Textdatei geschrieben werden.
Dieser Prozess wird wiederholt bis alle Bytes aus der zu durchsuchenden Datei gelesen wurden. Fortschritt wird dabei durch eine Leiste angezeigt, welche von einem dritten Thread auf die Befehlszeile geschrieben wird. 
Ist die Suche abgeschlossen wird noch die Anzahl der Ergebnisse angezeigt.

## Modul

Um das Kernelmodul zu installieren, muss die `rust_test.rs` Datei nach 
`/Pfad-zum-Kernel/samples/rust/` kopiert werden und anschließend müssen in diesem Ordner die Makefile und die Kconfig Datein
modifiziert werden, so dass diese das Modul enthalten. Mit 
`$ make menuconfig` kann das Modul dann unter 
`Kernel Hacking -> Samples -> Rust -> test device` aktiviert werden.

Nachdem der Kernel kompiliert wurde, startet das Modul beim nächsten Boot.

`$ make LLVM=1`

## Client

Um dem Client zu installieren, ist cargo im Grundverzeichnßis des Clients auszuführen:

`cargo build --release`

Die ausführare Datei befindet sich dann in `/Pfad-zum-Client/target/release/chrdevcli`


