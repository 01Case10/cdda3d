// CDDA Voxel Frontend — Phase 3 input injector. See voxel_input.h.
#include "voxel_input.h"

namespace voxel_ipc
{
namespace ipc = cdda::ipc;

void InputInjector::enqueue( const ipc::InputT &in )
{
    QueuedInput q;
    q.modifiers = in.modifiers;
    switch( in.kind ) {
        case ipc::InputKind_Key:
            q.kind = QueuedInput::Kind::KeyCode;
            q.code = in.key_code;
            break;
        case ipc::InputKind_Text:
            q.kind = QueuedInput::Kind::Text;
            q.text = in.text;
            break;
        case ipc::InputKind_MouseButton:
            q.kind = QueuedInput::Kind::MouseButton;
            q.mouse_x = in.mouse_x;
            q.mouse_y = in.mouse_y;
            q.button = in.button;
            q.pressed = in.pressed;
            break;
        case ipc::InputKind_MouseMove:
            q.kind = QueuedInput::Kind::MouseMove;
            q.mouse_x = in.mouse_x;
            q.mouse_y = in.mouse_y;
            break;
        case ipc::InputKind_Wheel:
            q.kind = QueuedInput::Kind::Wheel;
            q.mouse_x = in.mouse_x;
            q.mouse_y = in.mouse_y;
            q.wheel_delta = in.wheel_delta;
            break;
        case ipc::InputKind_Focus:
            return; // focus is engine-local state; nothing enters the game queue
    }
    queue_.push_back( std::move( q ) );
}

bool InputInjector::next( QueuedInput &out )
{
    if( queue_.empty() ) { return false; }
    out = std::move( queue_.front() );
    queue_.pop_front();
    return true;
}

} // namespace voxel_ipc
