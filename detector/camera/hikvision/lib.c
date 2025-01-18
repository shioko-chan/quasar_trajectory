#include "MvErrorDefine.h"
#include "MvCameraControl.h"

int main()
{
    int nRet = MV_OK;
    MV_CC_Initialize();
    MV_CC_Finalize();
    return nRet;
}