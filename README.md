# CellSight

- One side panel (left)
- One main viewport (center right)
- One top toolbar


## We need to

We need to create some reusable UI components for this application.

Text input, sliders, buttons, icon buttons, icon only buttons, annotation layer (for annotating the connected cameras visualized viewport), collapsible section.


---

## Side panel (left)

The side panel is composed of collapsible (animted) subsections
- Camera list (dropdown component)
- Capture and resolution and FPS (dropdown components)

## Sample CLI commands to retrieve the video stream on Linux:

---

❯ v4l2-ctl -d /dev/video0 --list-formats-ext
ioctl: VIDIOC_ENUM_FMT
        Type: Video Capture

        [0]: 'MJPG' (Motion-JPEG, compressed)
                Size: Discrete 2592x1944
                        Interval: Discrete 0.033s (30.000 fps)
                        Interval: Discrete 0.040s (25.000 fps)
                        Interval: Discrete 0.050s (20.000 fps)
                        Interval: Discrete 0.067s (15.000 fps)
                        Interval: Discrete 0.100s (10.000 fps)
                        Interval: Discrete 0.133s (7.500 fps)
                        Interval: Discrete 0.200s (5.000 fps)
                Size: Discrete 1280x960
                        Interval: Discrete 0.033s (30.000 fps)
                        Interval: Discrete 0.040s (25.000 fps)
                        Interval: Discrete 0.050s (20.000 fps)
                        Interval: Discrete 0.067s (15.000 fps)
                        Interval: Discrete 0.100s (10.000 fps)
                        Interval: Discrete 0.133s (7.500 fps)
                        Interval: Discrete 0.200s (5.000 fps)
                Size: Discrete 640x480
                        Interval: Discrete 0.033s (30.000 fps)
                        Interval: Discrete 0.040s (25.000 fps)
                        Interval: Discrete 0.050s (20.000 fps)
                        Interval: Discrete 0.067s (15.000 fps)
                        Interval: Discrete 0.100s (10.000 fps)
                        Interval: Discrete 0.133s (7.500 fps)
                        Interval: Discrete 0.200s (5.000 fps)

---

❯ mpv av://v4l2:/dev/video0 \
    --profile=low-latency \
    --demuxer-lavf-format=video4linux2
● Video  --vid=1  (mjpeg 2592x1944 30 fps)
VO: [gpu-next] 2592x1944 yuvj422p
