# OBS Filter
This was just a quick hack to see how the filter would look applied live to a video feed. Getting it compiling can be a bit of a hassle because building OBS plugins is a bit of a hassle.

## Before You Get Started
First this requires some patches to the `obs-wrapper` library to add support for video filters and once that's upstreamed it should be easier. See the patch in resources for the changes I'd made prior to any updates made during the PR.

## Building
These build steps focus on macOS because that's what I built this on. Linux should be similar with the obvious path changes and lack of codesigning requirements.

If using a local obs-wrapper checkout, I just copied the `libobs` folder (from the main obs repo) to the required folder (`rust-obs-plugins/obs-sys/obs`) to make building easier.

Then you'll need to point it to where your OBS install is for the libraries, on macOS this is /Applications/OBS.app/Contents/Frameworks.

### AppleSilicon
OBS is not (currently) released for Apple Silicon due to browser compatibility issues. This means that it runs through rosetta (you'll see this if you use Virtual Camera where it only shows up in apps *also* running through rosetta). This means that you cannot compile this filter natively, it needs to be xpiled to x86_64.

For compilation I ran with this: `LIBOBS_PATH=/Applications/OBS.app/Contents/Frameworks/  cargo build --release --target x86_64-apple-darwin` which should work on both AppleSilicon and Intel macOS machines.

## Signing
OBS is a sandboxed application thus enforces codesigning. Luckily, OBS has the entitlement to load dynamic libraries signed by other developers! Thus follow a standard codesigning pattern (no special entitlements are needed for the filter).

## Installing
Copy the signed binary to `/Applications/OBS.app/Contents/PlugIns/mac-halftonery.so` (it will not find it with the dylib extension in my experience).

## Running
Running OBS, it should pick up and load the code and allow you to apply it to video sources. Currently only the UYVY colour format is supported but adding others should not be too difficult as all that's needed is the conversion in and out of that format to the CMYK buffers that halftonery uses.