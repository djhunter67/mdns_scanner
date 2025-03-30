# mDns Network Scanner

This small project is a learning adventure in mDNS.  I scanned the network port on my home rig and was surprised at what was broadcasting mDNS packets on my network.

## Build -- Linux, Arch
- The two dependencies of this project are `extra/avahi` and `core/sqlite`. 
  - [mDNS Arch Wiki](https://wiki.archlinux.org/title/Avahi)
  - `yay -S avahi --noconfirm`
  - `sudo systemctl start avahi-daemon.service`
  - `yay -S sqlite --noconfirm`
  
- Next, you should be able to run the default program:
  - `cargo run`
  
- Expectation as of `version 0.1.0`

```bash
Initiating 1 second scan
        Discovered: EWS377APv3
        Discovered: HP LaserJet Pro MFP 3101-3108 [DA7C54]

Discovered 2 mDns devices

Name: HP LaserJet Pro MFP 3101-3108 [DA7C54]
IP: 10.20.20.50

Name: EWS377APv3
IP: 192.168.110.30
```


- Obviously, the devices you find on your network should be different.

