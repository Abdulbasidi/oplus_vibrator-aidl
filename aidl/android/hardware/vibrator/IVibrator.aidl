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

import android.hardware.vibrator.Braking;
import android.hardware.vibrator.CompositeEffect;
import android.hardware.vibrator.CompositePrimitive;
import android.hardware.vibrator.CompositePwleV2;
import android.hardware.vibrator.Effect;
import android.hardware.vibrator.EffectStrength;
import android.hardware.vibrator.FrequencyAccelerationMapEntry;
import android.hardware.vibrator.IVibratorCallback;
import android.hardware.vibrator.PrimitivePwle;
import android.hardware.vibrator.VendorEffect;

@VintfStability
interface IVibrator {

    const int CAP_ON_CALLBACK = 1 << 0;

    const int CAP_PERFORM_CALLBACK = 1 << 1;

    const int CAP_AMPLITUDE_CONTROL = 1 << 2;

    const int CAP_EXTERNAL_CONTROL = 1 << 3;

    const int CAP_EXTERNAL_AMPLITUDE_CONTROL = 1 << 4;

    const int CAP_COMPOSE_EFFECTS = 1 << 5;

    const int CAP_ALWAYS_ON_CONTROL = 1 << 6;

    const int CAP_GET_RESONANT_FREQUENCY = 1 << 7;

    const int CAP_GET_Q_FACTOR = 1 << 8;

    const int CAP_FREQUENCY_CONTROL = 1 << 9;

    const int CAP_COMPOSE_PWLE_EFFECTS = 1 << 10;

    const int CAP_PERFORM_VENDOR_EFFECTS = 1 << 11;

    const int CAP_COMPOSE_PWLE_EFFECTS_V2 = 1 << 12;

    int getCapabilities();

    void off();

    void on(in int timeoutMs, in @nullable IVibratorCallback callback);

    int perform(in Effect effect, in EffectStrength strength,
            in @nullable IVibratorCallback callback);

    Effect[] getSupportedEffects();

    void setAmplitude(in float amplitude);

    void setExternalControl(in boolean enabled);

    int getCompositionDelayMax();

    int getCompositionSizeMax();

    CompositePrimitive[] getSupportedPrimitives();

    int getPrimitiveDuration(CompositePrimitive primitive);

    void compose(in CompositeEffect[] composite, in @nullable IVibratorCallback callback);

    Effect[] getSupportedAlwaysOnEffects();

    void alwaysOnEnable(in int id, in Effect effect, in EffectStrength strength);

    void alwaysOnDisable(in int id);

    float getResonantFrequency();

    float getQFactor();

    float getFrequencyResolution();

    float getFrequencyMinimum();

    float[] getBandwidthAmplitudeMap();

    int getPwlePrimitiveDurationMax();

    int getPwleCompositionSizeMax();

    Braking[] getSupportedBraking();

    void composePwle(in PrimitivePwle[] composite, in @nullable IVibratorCallback callback);

    void performVendorEffect(
            in VendorEffect vendorEffect, in @nullable IVibratorCallback callback);

    List<FrequencyAccelerationMapEntry> getFrequencyToOutputAccelerationMap();

    int getPwleV2PrimitiveDurationMaxMillis();

    int getPwleV2CompositionSizeMax();

    int getPwleV2PrimitiveDurationMinMillis();

    void composePwleV2(
            in CompositePwleV2 composite, in @nullable IVibratorCallback callback);
}
