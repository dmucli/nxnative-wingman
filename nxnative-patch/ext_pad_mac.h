// An external controller over localhost UDP (Mac): DM_EXT_PAD=<port> makes the host listen on 127.0.0.1:<port> for a whole controller state that another
// process sends (a HOTAS mapper such as Wingman, which reads joysticks and throttles and turns them into Switch buttons and sticks). Included by front_mac.m,
// beside the raw HID pads (pad_hid_mac.h): the sender becomes a controller of its own at its first packet, through pad_join, so it takes a player slot as a
// connecting pad does (player 1 when no other controller holds it; a one-player game reads every player as player 1 either way).
//
// Packet (little endian, 36 bytes): "NXEP", u16 version (1), u16 flags (bit 0: the sender leaves, its slot is freed), u32 sequence, u64 buttons
// (dm_input.h bits: NpadButton 0..15, DM_BTN_SHAKE), i32 lx ly rx ry (±DM_STICK_MAX, y up). An older sequence than the last one is dropped.
// The host answers the sender's address about once a second with "NXEA", u16 version, u16 player (1-based, 0 while it has none), so the sender can show
// that the game is listening. With no packet for EXTPAD_STALE_MS the controller's state goes back to rest (nothing stays pressed when the sender stops or
// hangs); the slot stays held until a leave packet, so the player number does not move.
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <pthread.h>
#include <stdatomic.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#define EXTPAD_STALE_MS 300
static const char g_extpad_owner = 0;   // the slot table's owner key for the external controller
static _Atomic int g_extpad_player = -1;   // its slot, -1 while it has none (written on the main queue, read by the receiver)
static _Atomic int g_extpad_joining;
static int g_extpad_fd = -1;

static uint64_t extpad_ms(void) { struct timespec t; clock_gettime(CLOCK_MONOTONIC, &t); return (uint64_t)t.tv_sec * 1000 + (uint64_t)t.tv_nsec / 1000000; }
static int32_t extpad_axis(int32_t v) { return v > DM_STICK_MAX ? DM_STICK_MAX : v < -DM_STICK_MAX ? -DM_STICK_MAX : v; }
static uint16_t extpad_u16(const uint8_t *p) { return (uint16_t)(p[0] | p[1] << 8); }
static uint32_t extpad_u32(const uint8_t *p) { return (uint32_t)p[0] | (uint32_t)p[1] << 8 | (uint32_t)p[2] << 16 | (uint32_t)p[3] << 24; }

static void extpad_join(void) {   // main queue: the slot table belongs to it
    if (atomic_load(&g_extpad_player) < 0) {
        int p = pad_join(&g_extpad_owner, "External controller (DM_EXT_PAD)");
        atomic_store(&g_extpad_player, p);
    }
    atomic_store(&g_extpad_joining, 0);
}
static void extpad_leave(void) {   // main queue
    int p = atomic_exchange(&g_extpad_player, -1);
    if (p < 0) return;
    dm_pad_set_player(p, 0, 0, 0, 0, 0);
    pad_slot_free(&g_slots, &g_extpad_owner);
    dm_pad_players(pad_slot_mask(&g_slots));
    char nb[16]; npad_name(p, nb, sizeof nb); LOG("external controller left: player %d (npad %s) is free", p + 1, nb);
}

static void *extpad_main(void *arg) {
    (void)arg;
    uint8_t b[64]; struct sockaddr_in from; socklen_t fl;
    uint32_t last_seq = 0; int have_seq = 0, resting = 1; uint64_t last_rx = 0, last_ack = 0; int warned = 0;
    for (;;) {
        fl = sizeof from;
        ssize_t n = recvfrom(g_extpad_fd, b, sizeof b, 0, (struct sockaddr *)&from, &fl);
        uint64_t now = extpad_ms();
        int p = atomic_load(&g_extpad_player);
        if (n < 0) {   // the receive timeout: go back to rest when the sender went quiet
            if (!resting && p >= 0 && now - last_rx > EXTPAD_STALE_MS) { dm_pad_set_player(p, 0, 0, 0, 0, 0); resting = 1; have_seq = 0; LOG("external controller: no input for %d ms, released", EXTPAD_STALE_MS); }
            continue;
        }
        if (n < 36 || memcmp(b, "NXEP", 4) || extpad_u16(b + 4) != 1) { if (!warned++) LOG("external controller: a packet of %zd bytes that is not NXEP version 1, ignored", n); continue; }
        uint16_t flags = extpad_u16(b + 6); uint32_t seq = extpad_u32(b + 8);
        if (flags & 1) { dispatch_async(dispatch_get_main_queue(), ^{ extpad_leave(); }); resting = 1; have_seq = 0; continue; }
        if (have_seq && (int32_t)(seq - last_seq) <= 0 && last_seq - seq < 1000) continue;   // reordered or repeated (a restarted sender starts low: more than 1000 back is taken)
        last_seq = seq; have_seq = 1; last_rx = now;
        if (p < 0) {
            if (!atomic_exchange(&g_extpad_joining, 1)) dispatch_async(dispatch_get_main_queue(), ^{ extpad_join(); });
            continue;   // the first packets only ask for a slot; the sender sends 100s a second, so the next one plays
        }
        uint64_t btn = (uint64_t)extpad_u32(b + 12) | (uint64_t)extpad_u32(b + 16) << 32;
        btn &= 0xffffull | DM_BTN_SHAKE;
        int32_t a[4]; for (int i = 0; i < 4; i++) a[i] = extpad_axis((int32_t)extpad_u32(b + 20 + 4 * i));
        dm_pad_set_player(p, btn, a[0], a[1], a[2], a[3]);
        if (resting) { char nb[16]; npad_name(p, nb, sizeof nb); LOG("external controller: playing as player %d (npad %s)", p + 1, nb); resting = 0; }
        if (now - last_ack >= 1000) {
            uint8_t ack[8] = {'N', 'X', 'E', 'A', 1, 0, (uint8_t)(p + 1), 0};
            sendto(g_extpad_fd, ack, sizeof ack, 0, (struct sockaddr *)&from, fl);
            last_ack = now;
        }
    }
    return NULL;
}

static void extpad_start(void) {
    const char *e = getenv("DM_EXT_PAD"); if (!e || !*e) return;
    int port = atoi(e);
    if (port <= 0 || port > 65535) { LOG("external controller: DM_EXT_PAD=%s is not a port number, off", e); return; }
    g_extpad_fd = socket(AF_INET, SOCK_DGRAM, 0);
    if (g_extpad_fd < 0) { LOG("external controller: no socket, off"); return; }
    struct sockaddr_in a = {0}; a.sin_family = AF_INET; a.sin_port = htons((uint16_t)port); a.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    int one = 1; setsockopt(g_extpad_fd, SOL_SOCKET, SO_REUSEADDR, &one, sizeof one);
    if (bind(g_extpad_fd, (struct sockaddr *)&a, sizeof a) < 0) { LOG("external controller: port %d is in use, off", port); close(g_extpad_fd); g_extpad_fd = -1; return; }
    struct timeval tv = {0, 100 * 1000}; setsockopt(g_extpad_fd, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof tv);
    pthread_t t; pthread_create(&t, NULL, extpad_main, NULL); pthread_detach(t);
    LOG("external controller: listening on 127.0.0.1:%d (DM_EXT_PAD)", port);
}
