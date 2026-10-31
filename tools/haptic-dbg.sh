
set -e

TAG="haptic-dbg"

log() { echo "[$TAG] $*"; }
err() { echo "[$TAG] ERROR: $*" >&2; exit 1; }

find_haptic() {
    for dev in /sys/bus/i2c/devices/[0-9]*-00*/; do
        [ -f "$dev/name" ] || continue
        name=$(cat "$dev/name" 2>/dev/null)
        if echo "$name" | grep -qi "awinic_haptic"; then
            echo "$dev"
            return 0
        fi
    done
    return 1
}

HAPTIC_PATH="${HAPTIC_PATH:-$(find_haptic 2>/dev/null)}"

resolve_path() {
    if [ -z "$HAPTIC_PATH" ]; then
        HAPTIC_PATH=$(find_haptic 2>/dev/null) || err "awinic_haptic not found on any I2C bus"
    fi
}

w() {
    resolve_path
    local node="$HAPTIC_PATH$1"
    [ -f "$node" ] || err "node not found: $node"
    echo "$2" > "$node" 2>/dev/null
    log "w $1 <- $2"
}

r() {
    resolve_path
    local node="$HAPTIC_PATH$1"
    [ -f "$node" ] || err "node not found: $node"
    cat "$node" 2>/dev/null
}

cmd_scan() {
    log "Scanning I2C bus for awinic_haptic..."
    for dev in /sys/bus/i2c/devices/[0-9]*-00*/; do
        [ -f "$dev/name" ] || continue
        name=$(cat "$dev/name" 2>/dev/null)
        log "  $dev -> name=$name"
        if echo "$name" | grep -qi "awinic_haptic"; then
            log "  ^^^ MATCH FOUND"
        fi
    done
    echo
    log "HAPTIC_PATH=$HAPTIC_PATH"
}

cmd_status() {
    resolve_path
    echo "=== Haptic Driver State ==="
    echo "Path:    $HAPTIC_PATH"
    echo "Name:    $(r name)"
    echo
    echo "--- Playback ---"
    echo "activate:       $(r activate)"
    echo "activate_mode:  $(r activate_mode)"
    echo "effect_id:      $(r effect_id)"
    echo "duration:       $(r duration)"
    echo
    echo "--- Drive Parameters ---"
    echo "vmax:           $(r vmax)"
    echo "gain:           $(r gain)"
    echo "auto_boost:     $(r auto_boost)"
    echo "ram_vbat_comp:  $(r ram_vbat_comp)"
    echo
    echo "--- Firmware ---"
    for f in fw_name fw_version reg; do
        [ -f "$HAPTIC_PATH$f" ] && echo "$f: $(r $f)"
    done
    echo
    echo "--- Other ---"
    for f in lra_f0 lra_f0_detected rtp_input ram_input di_cont; do
        [ -f "$HAPTIC_PATH$f" ] && echo "$f: $(r $f)"
    done
}

cmd_map() {
    echo "=== Effect ID Mapping (hoshikv-haptic-rs) ==="
    echo
    echo "effect_id routing (by base amplitude raw):"
    echo "  base <= 0xAAAA (43690) -> motor effect_id 2  (light tick waveform)"
    echo "  base >= 0xEEEE (61166) -> motor effect_id 7  (heavy click waveform)"
    echo "  otherwise              -> motor effect_id 1  (default click waveform)"
    echo
    echo "Predefined effects:"
    echo "  Effect           dur_ms   base_raw   motor_id   scale(STRONG)"
    echo "  --------         ------   --------   --------   -------------"
    echo "  CLICK             60      0xFFFF     1          1.0"
    echo "  DOUBLE_CLICK     130      0xFFFF     1          1.0"
    echo "  TICK              25      0xAAAA     2          1.0"
    echo "  THUD             110      0xFFFF     1          1.0"
    echo "  POP               35      0xFFFF     1          1.0"
    echo "  HEAVY_CLICK       80      0xFFFF     7          1.0"
    echo "  TEXTURE_TICK      12      0x7777     2          1.0"
    echo
    echo "EffectStrength scaling:"
    echo "  LIGHT  = 0.6  -> amplitude 0.6"
    echo "  MEDIUM = 0.85 -> amplitude 0.85"
    echo "  STRONG = 1.0  -> amplitude 1.0"
    echo
    echo "Tap mode amplitude -> hw:"
    echo "  t = clamp(amplitude, 0, 2) / 2"
    echo "  vmax = 0x28 + (0x50 - 0x28) * t^2   (40 -> 80)"
    echo "  gain = 1 + (0xB4 - 1) * t^2           (1 -> 180)"
    echo
    echo "Composite primitives (compose):"
    echo "  NOOP=0(0ms)  CLICK=1(60ms)  THUD=2(110ms)  SPIN=3(200ms)"
    echo "  QUICK_RISE=4(50ms)  SLOW_RISE=5(120ms)  QUICK_FALL=6(80ms)"
    echo "  LIGHT_TICK=7(15ms)  LOW_TICK=8(30ms)"
}

cmd_play() {
    local eid="${1:-1}"
    local dur="${2:-60}"
    local vmax="${3:-0x50}"
    local gain="${4:-0xB4}"

    resolve_path
    log "Playing effect_id=$eid dur=${dur}ms vmax=$vmax gain=$gain"
    w effect_id "$eid"
    w activate_mode 0
    w vmax "$vmax"
    w gain "$gain"
    w activate 1
    sleep "$(echo "scale=3; $dur / 1000" | bc 2>/dev/null || echo "0.06")"
    w activate 0
    log "Done"
}

cmd_tap() {
    local amp="${1:-1.0}"
    local dur="${2:-60}"

    resolve_path
    local t_int
    t_int=$(echo "scale=4; if ($amp > 2) 2; if ($amp < 0) 0; $amp" | bc 2>/dev/null || echo "$amp")
    local vmax_dec gain_dec
    vmax_dec=$(echo "scale=0; 40 + (80 - 40) * $t_int * $t_int / 1" | bc 2>/dev/null || echo "80")
    gain_dec=$(echo "scale=0; 1 + (180 - 1) * $t_int * $t_int / 1" | bc 2>/dev/null || echo "180")

    [ "$vmax_dec" -gt 80 ] 2>/dev/null && vmax_dec=80
    [ "$gain_dec" -gt 180 ] 2>/dev/null && gain_dec=180

    local vmax_hex=$(printf "0x%02X" "$vmax_dec")
    local gain_hex=$(printf "0x%02X" "$gain_dec")

    log "Tap: amp=$amp dur=${dur}ms -> vmax=$vmax_hex gain=$gain_hex"

    local base=$((amp * 65535 / 1))
    local eid=1
    if [ "$base" -le 43690 ] 2>/dev/null; then
        eid=2
    elif [ "$base" -ge 61166 ] 2>/dev/null; then
        eid=7
    fi

    w effect_id "$eid"
    w activate_mode 0
    w vmax "$vmax_hex"
    w gain "$gain_hex"
    w activate 1
    sleep "$(echo "scale=3; $dur / 1000" | bc 2>/dev/null || echo "0.06")"
    w activate 0
    log "Done (eid=$eid)"
}

cmd_loop() {
    local dur="${1:-2000}"
    local amp="${2:-1.0}"

    resolve_path
    local gain_dec
    local amp2
    amp2=$(echo "scale=4; $amp * 2" | bc 2>/dev/null || echo "2")
    [ $(echo "$amp2 > 2" | bc 2>/dev/null || echo "0") -eq 1 ] && amp2=2
    gain_dec=$(echo "scale=0; 1 + (180 - 1) * $amp2 * $amp2 / 1" | bc 2>/dev/null || echo "180")
    [ "$gain_dec" -gt 180 ] 2>/dev/null && gain_dec=180
    local gain_hex=$(printf "0x%02X" "$gain_dec")

    log "Loop: dur=${dur}ms amp=$amp -> gain=$gain_hex"
    w activate_mode 3
    w vmax 0x50
    w gain "$gain_hex"
    w duration "$dur"
    w activate 1
    log "Motor running. Use 'off' to stop."
}

cmd_off() {
    resolve_path
    w activate 0
    log "Motor stopped"
}

cmd_amplitude() {
    local amp="${1:-1.0}"
    local dur="${2:-60}"
    resolve_path

    local t_int
    t_int=$(echo "scale=4; if ($amp > 2) 2; if ($amp < 0) 0; $amp" | bc 2>/dev/null || echo "$amp")
    local vmax_dec gain_dec
    vmax_dec=$(echo "scale=0; 40 + (80 - 40) * $t_int * $t_int / 1" | bc 2>/dev/null || echo "80")
    gain_dec=$(echo "scale=0; 1 + (180 - 1) * $t_int * $t_int / 1" | bc 2>/dev/null || echo "180")
    [ "$vmax_dec" -gt 80 ] 2>/dev/null && vmax_dec=80
    [ "$gain_dec" -gt 180 ] 2>/dev/null && gain_dec=180

    local vmax_hex=$(printf "0x%02X" "$vmax_dec")
    local gain_hex=$(printf "0x%02X" "$gain_dec")

    local base=$(echo "scale=0; $amp * 65535 / 1" | bc 2>/dev/null || echo "43690")
    local eid=1
    [ "$base" -le 43690 ] 2>/dev/null && eid=2
    [ "$base" -ge 61166 ] 2>/dev/null && eid=7

    log "Amplitude test: amp=$amp base=$base eid=$eid vmax=$vmax_hex gain=$gain_hex dur=${dur}ms"
    w effect_id "$eid"
    w activate_mode 0
    w vmax "$vmax_hex"
    w gain "$gain_hex"
    w activate 1
    sleep "$(echo "scale=3; $dur / 1000" | bc 2>/dev/null || echo "0.06")"
    w activate 0
    log "Done"
}

cmd_sweep() {
    resolve_path
    log "=== Sweeping all effect_ids with max params ==="
    for eid in 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 21; do
        log "--- effect_id=$eid ---"
        w effect_id "$eid"
        w activate_mode 0
        w vmax 0x50
        w gain 0xB4
        w activate 1
        sleep 0.1
        w activate 0
        sleep 0.15
    done
    log "Sweep done"
}

case "${1:-status}" in
    scan)       cmd_scan ;;
    status|s)   cmd_status ;;
    map|m)      cmd_map ;;
    play|p)     cmd_play "$2" "$3" "$4" "$5" ;;
    tap|t)      cmd_tap "$2" "$3" ;;
    loop|l)     cmd_loop "$2" "$3" ;;
    off|stop)   cmd_off ;;
    amplitude|a) cmd_amplitude "$2" "$3" ;;
    sweep|sw)   cmd_sweep ;;
    *)
        echo "Usage: hoshikv-haptic-dbg <command> [args...]"
        echo
        echo "Commands:"
        echo "  (none)/status   Dump driver state"
        echo "  scan            Find haptic sysfs path"
        echo "  map             Show effect_id mapping"
        echo "  play <id> [dur] [vmax] [gain]   Play raw effect"
        echo "  tap <amp> [dur] Play click at amplitude 0-2.0"
        echo "  loop <dur> [amp]  Continuous vibration"
        echo "  amplitude <amp> [dur]  Play click with HAL amplitude curve"
        echo "  sweep           Play all effect_ids"
        echo "  off             Stop motor"
        ;;
esac
