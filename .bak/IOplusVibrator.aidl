package vendor.oplus.hardware.oplusvibrator;

import vendor.aac.hardware.richtap.vibrator.IRichtapCallback;

interface IOplusVibrator {
    void init(IRichtapCallback callback);
    void setDynamicScale(int scale, IRichtapCallback callback);
    void setF0(int f0, IRichtapCallback callback);
    void stop(IRichtapCallback callback);
    void setAmplitude(int amplitude, IRichtapCallback callback);
    void performHeParam(int a, int b, int c, IRichtapCallback callback);
    void off(IRichtapCallback callback);
    void on(int amplitude, IRichtapCallback callback);
    int perform(int effectId, byte strength, IRichtapCallback callback);
    void performHe(int a, int b, int c, int d, int[] e, IRichtapCallback callback);
    void setHapticParam(int[] params, int length, IRichtapCallback callback);
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
    void writeHapticAudio(int[] data);
    void setVibratorTouchStyle(int style);
    int getVibratorTouchStyle();
    void setPowerOnVibratorSwitch(String switchStr);
}
