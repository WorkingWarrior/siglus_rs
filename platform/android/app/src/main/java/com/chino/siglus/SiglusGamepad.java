package com.chino.siglus;

import android.util.SparseIntArray;
import android.view.InputDevice;
import android.view.KeyEvent;
import android.view.MotionEvent;

/** Converts Android controller input to the engine's zero-based joypad buttons. */
final class SiglusGamepad {
    interface ButtonListener {
        void onButton(int button, boolean down, boolean cancelled);
    }

    private static final float AXIS_PRESS_THRESHOLD = 0.55f;
    private static final float AXIS_RELEASE_THRESHOLD = 0.40f;

    private static final int L2 = 8;
    private static final int R2 = 9;
    private static final int DPAD_LEFT = 12;
    private static final int LEFT_STICK_LEFT = 16;
    private static final int RIGHT_STICK_LEFT = 20;

    private final ButtonListener listener;
    // Keep key and axis sources separate: controllers can report both for one control.
    private final SparseIntArray keys = new SparseIntArray();
    private final SparseIntArray axes = new SparseIntArray();
    private int held;

    SiglusGamepad(ButtonListener listener) {
        this.listener = listener;
    }

    boolean onKey(KeyEvent event) {
        if (!isControllerEvent(event)) {
            return false;
        }

        int button = buttonForKeyCode(event.getKeyCode());
        int action = event.getAction();
        if (button < 0 || (action != KeyEvent.ACTION_DOWN && action != KeyEvent.ACTION_UP)) {
            return false;
        }
        if (event.getRepeatCount() == 0 || action == KeyEvent.ACTION_UP) {
            int deviceId = event.getDeviceId();
            int previous = keys.get(deviceId);
            int bit = 1 << button;
            int buttons = action == KeyEvent.ACTION_DOWN ? previous | bit : previous & ~bit;
            setButtons(keys, deviceId, buttons, event.isCanceled());
        }
        return true;
    }

    boolean onMotion(MotionEvent event) {
        if (!event.isFromSource(InputDevice.SOURCE_JOYSTICK)
                || event.getActionMasked() != MotionEvent.ACTION_MOVE) {
            return false;
        }

        int deviceId = event.getDeviceId();
        int prev = axes.get(deviceId);
        int buttons = directions(event, MotionEvent.AXIS_HAT_X, MotionEvent.AXIS_HAT_Y, DPAD_LEFT, prev)
                | directions(event, MotionEvent.AXIS_X, MotionEvent.AXIS_Y, LEFT_STICK_LEFT, prev)
                | directions(event, MotionEvent.AXIS_Z, MotionEvent.AXIS_RZ, RIGHT_STICK_LEFT, prev);
        if (axisActive(Math.max(event.getAxisValue(MotionEvent.AXIS_LTRIGGER),
                event.getAxisValue(MotionEvent.AXIS_BRAKE)), (prev & (1 << L2)) != 0)) {
            buttons |= 1 << L2;
        }
        if (axisActive(Math.max(event.getAxisValue(MotionEvent.AXIS_RTRIGGER),
                event.getAxisValue(MotionEvent.AXIS_GAS)), (prev & (1 << R2)) != 0)) {
            buttons |= 1 << R2;
        }
        setButtons(axes, deviceId, buttons, false);
        return true;
    }

    void removeDevice(int deviceId) {
        keys.delete(deviceId);
        axes.delete(deviceId);
        updateHeld(true);
    }

    void releaseAll() {
        keys.clear();
        axes.clear();
        updateHeld(true);
    }

    private void setButtons(SparseIntArray source, int deviceId, int buttons, boolean cancelled) {
        if (source.get(deviceId) == buttons) {
            return;
        }
        if (buttons == 0) {
            source.delete(deviceId);
        } else {
            source.put(deviceId, buttons);
        }
        updateHeld(cancelled);
    }

    private void updateHeld(boolean cancelled) {
        int buttons = 0;
        for (int i = 0; i < keys.size(); i++) {
            buttons |= keys.valueAt(i);
        }
        for (int i = 0; i < axes.size(); i++) {
            buttons |= axes.valueAt(i);
        }
        int released = held & ~buttons;
        int pressed = buttons & ~held;
        held = buttons;
        sendEdges(released, false, cancelled);
        sendEdges(pressed, true, false);
    }

    private void sendEdges(int buttons, boolean down, boolean cancelled) {
        while (buttons != 0) {
            int button = Integer.numberOfTrailingZeros(buttons);
            listener.onButton(button, down, cancelled);
            buttons &= ~(1 << button);
        }
    }

    private static boolean isControllerEvent(KeyEvent event) {
        if (event.isFromSource(InputDevice.SOURCE_GAMEPAD)
                || event.isFromSource(InputDevice.SOURCE_JOYSTICK)) {
            return true;
        }
        if (!event.isFromSource(InputDevice.SOURCE_DPAD)) {
            return false;
        }
        InputDevice device = event.getDevice();
        return device != null
                && (device.supportsSource(InputDevice.SOURCE_GAMEPAD)
                        || device.supportsSource(InputDevice.SOURCE_JOYSTICK));
    }

    private static boolean axisActive(float value, boolean wasActive) {
        float threshold = wasActive ? AXIS_RELEASE_THRESHOLD : AXIS_PRESS_THRESHOLD;
        return value > threshold;
    }

    private static int directions(MotionEvent event, int xAxis, int yAxis, int left, int prevButtons) {
        float x = event.getAxisValue(xAxis);
        float y = event.getAxisValue(yAxis);
        int buttons = 0;
        if (axisActive(-x, (prevButtons & (1 << left)) != 0)) buttons |= 1 << left;
        if (axisActive(-y, (prevButtons & (1 << (left + 1))) != 0)) buttons |= 1 << (left + 1);
        if (axisActive(x, (prevButtons & (1 << (left + 2))) != 0)) buttons |= 1 << (left + 2);
        if (axisActive(y, (prevButtons & (1 << (left + 3))) != 0)) buttons |= 1 << (left + 3);
        return buttons;
    }

    private static int buttonForKeyCode(int keyCode) {
        switch (keyCode) {
            case KeyEvent.KEYCODE_BUTTON_A:
            case KeyEvent.KEYCODE_DPAD_CENTER: return 0;
            case KeyEvent.KEYCODE_BUTTON_B: return 1;
            case KeyEvent.KEYCODE_BUTTON_X: return 2;
            case KeyEvent.KEYCODE_BUTTON_Y: return 3;
            case KeyEvent.KEYCODE_BUTTON_THUMBL: return 4;
            case KeyEvent.KEYCODE_BUTTON_THUMBR: return 5;
            case KeyEvent.KEYCODE_BUTTON_L1: return 6;
            case KeyEvent.KEYCODE_BUTTON_R1: return 7;
            case KeyEvent.KEYCODE_BUTTON_L2: return L2;
            case KeyEvent.KEYCODE_BUTTON_R2: return R2;
            case KeyEvent.KEYCODE_BUTTON_START: return 10;
            case KeyEvent.KEYCODE_BUTTON_SELECT: return 11;
            case KeyEvent.KEYCODE_DPAD_LEFT: return DPAD_LEFT;
            case KeyEvent.KEYCODE_DPAD_UP: return DPAD_LEFT + 1;
            case KeyEvent.KEYCODE_DPAD_RIGHT: return DPAD_LEFT + 2;
            case KeyEvent.KEYCODE_DPAD_DOWN: return DPAD_LEFT + 3;
            default: return -1;
        }
    }
}
