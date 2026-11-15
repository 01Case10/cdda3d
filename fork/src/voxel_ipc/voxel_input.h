// CDDA Voxel Frontend — Phase 3: input injector.
//
// Engine Input messages (schema) are translated into the game's own
// input path: the real adapter converts each queued event into an
// `input_event` (src/input_enums.h:63 — type + sequence + mouse_pos +
// text + modifiers) and pushes it onto the same queue that
// `input_context::get_raw_input()` (src/input_context.h:427) drains, so
// keybindings, contexts and menus work unchanged (recon test 0.7 shape).
// Mouse coordinates arrive in display-buffer pixels; converting them to
// cells is the injector's job (the SDL backend does the equivalent today
// via window_to_display_buffer_coords, sdltiles.cpp).
#pragma once

#include <cstdint>
#include <deque>
#include <string>

#include "cdda_ipc_generated.h"

namespace voxel_ipc
{

// Game-API-agnostic queued event (the real adapter maps it to input_event).
struct QueuedInput {
    enum class Kind { KeyCode, Char, Text, MouseButton, MouseMove, Wheel } kind = Kind::KeyCode;
    int code = 0;              // keycode (input.h keycode::*) or char32 for Char
    std::string text;          // Text events (UTF-8)
    int mouse_x = 0, mouse_y = 0; // display-buffer pixels
    int button = -1;
    bool pressed = false;
    int wheel_delta = 0;
    uint32_t modifiers = 0;    // schema bitmask: 1 shift, 2 ctrl, 4 alt, 8 meta
};

class InputInjector
{
    public:
        // Translate one wire Input into queued game events.
        void enqueue( const cdda::ipc::InputT &in );
        bool empty() const { return queue_.empty(); }
        size_t size() const { return queue_.size(); }
        // Pop the next event (the real get_raw_input hook calls this).
        bool next( QueuedInput &out );
        void clear() { queue_.clear(); }
    private:
        std::deque<QueuedInput> queue_;
};

} // namespace voxel_ipc
