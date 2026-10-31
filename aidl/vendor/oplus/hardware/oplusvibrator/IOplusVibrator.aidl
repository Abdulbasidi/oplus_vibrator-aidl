package vendor.oplus.hardware.oplusvibrator;

import android.os.ParcelFileDescriptor;
import vendor.aac.hardware.richtap.vibrator.IRichtapCallback;

// Method order == transaction order (FIRST_CALL_TRANSACTION + index) and must
// match the IOplusVibrator V2 compiled into the ROM framework (oplus-services.jar):
//   1 init            2 setDynamicScale   3 setF0              4 stop
//   5 setAmplitude    6 performHeParam    7 off                8 on
//   9 perform        10 performEnvelope  11 performRtp        12 performHe
//  13 setHapticParam 14 linearMotorVibratorOff 15 linearMotorVibratorOn
//  16 startCalibrate 17 getCalibrateResults 18 getStatus      19 setVmax
//  20 setMotorOldTest 21 getSupportFeatures 22 writeGunType   23 writeGunMode
//  24 writeBulletNum 25 writeHapticAudio 26 setVibratorTouchStyle
//  27 getVibratorTouchStyle 28 setPowerOnVibratorSwitch
// Do NOT reorder or insert methods in the middle.
//
// In the framework init/off/on/setAmplitude/setDynamicScale/setF0/setHapticParam
// and performEnvelope/performRtp/performHe are oneway; that is a client-side flag
// and the server side does not need it.
interface IOplusVibrator {
    void init(@nullable IRichtapCallback callback);
    void setDynamicScale(int scale, @nullable IRichtapCallback callback);
    void setF0(int f0, @nullable IRichtapCallback callback);
    void stop(@nullable IRichtapCallback callback);
    void setAmplitude(int amplitude, @nullable IRichtapCallback callback);
    void performHeParam(int a, int b, int c, @nullable IRichtapCallback callback);
    void off(@nullable IRichtapCallback callback);
    void on(int amplitude, @nullable IRichtapCallback callback);
    int perform(int effectId, byte strength, @nullable IRichtapCallback callback);
    void performEnvelope(in int[] envInfo, boolean fastFlag, @nullable IRichtapCallback callback);
    void performRtp(in ParcelFileDescriptor fd, @nullable IRichtapCallback callback);
    void performHe(int a, int b, int c, int d, int[] e, @nullable IRichtapCallback callback);
    void setHapticParam(int[] params, int length, @nullable IRichtapCallback callback);
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
    void writeHapticAudio(String data);
    void setVibratorTouchStyle(int style);
    int getVibratorTouchStyle();
    void setPowerOnVibratorSwitch(String switchStr);
}
