package vendor.oplus.hardware.oplusvibrator;

import android.os.ParcelFileDescriptor;
import vendor.aac.hardware.richtap.vibrator.IRichtapCallback;

// Must match the interface shipped in COS17 oplus-services.jar
// (VERSION = 2, HASH = b103e6d812aa472b9c5b7deefd7986fa729ce1a8).
// Transaction codes are assigned by declaration order, so do not reorder.
interface IOplusVibrator {
    oneway void init(@nullable IRichtapCallback callback);
    oneway void setDynamicScale(int scale, @nullable IRichtapCallback callback);
    oneway void setF0(int f0, @nullable IRichtapCallback callback);
    void stop(@nullable IRichtapCallback callback);
    oneway void setAmplitude(int amplitude, @nullable IRichtapCallback callback);
    void performHeParam(int a, int b, int c, @nullable IRichtapCallback callback);
    oneway void off(@nullable IRichtapCallback callback);
    oneway void on(int amplitude, @nullable IRichtapCallback callback);
    int perform(int effectId, byte strength, @nullable IRichtapCallback callback);
    oneway void performEnvelope(in int[] envInfo, boolean fastFlag, @nullable IRichtapCallback callback);
    oneway void performRtp(in ParcelFileDescriptor hdl, @nullable IRichtapCallback callback);
    oneway void performHe(int a, int b, int c, int d, in int[] e, @nullable IRichtapCallback callback);
    oneway void setHapticParam(in int[] params, int length, @nullable IRichtapCallback callback);
    void linearMotorVibratorOff();
    void linearMotorVibratorOn(int duration, int strength, boolean extra);
    void startCalibrate();
    int[] getCalibrateResults();
    int getStatus();
    void setVmax(int strength);
    void setMotorOldTest(int test);
    byte[] getSupportFeatures();
    void writeGunType(int type);
    void writeGunMode(int mode);
    void writeBulletNum(int num);
    void writeHapticAudio(String buffer);
    void setVibratorTouchStyle(int style);
    int getVibratorTouchStyle();
    void setPowerOnVibratorSwitch(String switchStr);
}
