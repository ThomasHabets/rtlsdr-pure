# rtlsdr-pure

Pure Rust implementation of librtlsdr

<https://github.com/ThomasHabets/rtlsdr-pure>

## Purpose

Makes it easier and cleaner to access RTL SDR from the browser.

## Status

Vibe coded test. The LLM took more than a sneak peak at
<https://github.com/osmocom/rtl-sdr/>, so this is a derived work of that, with
the same GPL license.

## If you get error claiming interface, in the browser

```
USB communication failed: NetworkError: Failed to execute 'claimInterface' on 'USBDevice': Unable to claim interface"
```

Then the kernel may have claimed it for the DVB driver. You can either unload
the DVB kernel modules (if they're modules), or just tell the driver to unbind
it.

You can find the device path with:

```
$ lsusb -t
Bus 001.[…]
    |__ Port 003: Dev 061, If 0, Class=Vendor Specific Class, Driver=dvb_usb_rtl28xxu, 480M
```

The format is `<bus-port>:<config>.<interface>`.

```
echo '1-3:1.0' | sudo tee /sys/bus/usb/drivers/dvb_usb_rtl28xxu/unbind
```
