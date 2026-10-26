/*
 * Copyright (C) 2019 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package android.hardware.vibrator;

@VintfStability
@Backing(type="int")
enum CompositePrimitive {

    NOOP = 0,

    CLICK = 1,

    THUD = 2,

    SPIN = 3,

    QUICK_RISE = 4,

    SLOW_RISE = 5,

    QUICK_FALL = 6,

    LIGHT_TICK = 7,

    LOW_TICK = 8,
}
