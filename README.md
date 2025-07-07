# VRLH Power Manager

Cross-platform program to power on and off your virtual reality lighthouses.

## Linux

SteamVR on Linux does not automatically power your base stations on and off. Rather than turning them off by physically unplugging the power, you can use this program to shut off and turn on your base stations yourself.

## Windows

It's been claimed that if you use virtual reality daily, it's better to leave your base stations running constantly. This avoids the wear and tear caused by the motor stopping and stopping.

Rather than depending on SteamVR to automatically start and stop your base stations, this program gives you the option to turn them on/off manually.

## Hardware Compatibility

This program only works with the following base stations:

- [Valve Index Base Station](https://store.steampowered.com/app/1059570/Valve_Index_Base_Station/)
- [HTC SteamVR Base Station 2.0](https://www.vive.com/us/accessory/base-station2/)

## Alternative Programs

If this program doesn't suit your needs, one of these might treat you better:

- [lighthouse_pm](https://github.com/jeroen1602/lighthouse_pm) (Android App)
- [lighthouse-v2-manager](https://github.com/nouser2013/lighthouse-v2-manager) (Python Script)
- [lhctrl](https://github.com/risa2000/lhctrl) (Python Script)
- [lh2ctrl](https://github.com/risa2000/lh2ctrl) (Python Script)

## For Developers

On Ubuntu, use the following command to install the required dependencies:

```sh
sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev libdbus-1-dev pkg-config
```
