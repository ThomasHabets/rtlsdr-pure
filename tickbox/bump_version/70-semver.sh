#!/usr/bin/env bash
exit 0
exec cargo semver-checks --only-explicit-features --features rtlsdr,soapysdr,fast-math,audio,fftw,async,pipewire,volk
