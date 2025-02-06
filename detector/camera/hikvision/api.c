#include "MvErrorDefine.h"
#include "MvCameraControl.h"
#include "CameraParams.h"
#include <stdlib.h>
#include <string.h>
#include "../camera_utils.h"

typedef struct
{
    void *handle;
    MV_FRAME_OUT frame;
} camera;
/**
 * @brief API全局状态结构体
 * @param sdk_initialized 该字段指示是否已经完成SDK的初始化
 * @param device_enumerated 该字段指示是否已经完成设备枚举
 * @param device_list 该字段存储设备枚举列表
 */
static struct
{
    bool sdk_initialized;
    camera *cam_list;
    // void **cam_handle_list;
    MV_CC_DEVICE_INFO_LIST device_list;
} API_STATE = {0};

inline static bool check_hik_err(api_error *ret, int err)
{
    if (err != MV_OK)
    {
        ret->is_hik_err = true;
        ret->code = err;
        return false;
    }
    return true;
}

api_error uninitialize_camera()
{
    api_error ret = {false, MV_OK};
    if (API_STATE.cam_list != NULL)
    {
        for (unsigned int i = 0; i < API_STATE.device_list.nDeviceNum; i++)
        {
            check_hik_err(&ret, MV_CC_CloseDevice(API_STATE.cam_list[i].handle));
            check_hik_err(&ret, MV_CC_DestroyHandle(API_STATE.cam_list[i].handle));
        }
        free(API_STATE.cam_list);
    }
    memset(&API_STATE.device_list, 0, sizeof(MV_CC_DEVICE_INFO_LIST));
    API_STATE.cam_list = NULL;

    return ret;
}

api_error init()
{
    api_error ret = {false, MV_OK};
    if (API_STATE.sdk_initialized)
    {
        ret.code = CAMERA_API_ALREADY_INITIALIZED;
        return ret;
    }
    if (check_hik_err(&ret, MV_CC_Initialize()))
    {
        API_STATE.sdk_initialized = true;
    }
    return ret;
}

api_error final()
{
    if (!API_STATE.sdk_initialized)
    {
        api_error ret = {false, CAMERA_API_NOT_INITIALIZED};
        return ret;
    }

    api_error ret = uninitialize_camera();
    if (ret.code != MV_OK)
    {
        return ret;
    }

    if (check_hik_err(&ret, MV_CC_Finalize()))
    {
        API_STATE.sdk_initialized = false;
    }
    return ret;
}

api_error enumerate_devices(unsigned int *device_num)
{
    if (!API_STATE.sdk_initialized)
    {
        api_error ret = {false, CAMERA_API_NOT_INITIALIZED};
        return ret;
    }

    api_error ret = {false, MV_OK};
    if (API_STATE.device_list.nDeviceNum > 0)
    {
        ret = uninitialize_camera();
    }
    memset(&API_STATE.device_list, 0, sizeof(MV_CC_DEVICE_INFO_LIST));
    if (!check_hik_err(&ret, MV_CC_EnumDevices(MV_USB_DEVICE, &API_STATE.device_list))) // 仅考虑USB相机
    {
        return ret;
    }

    API_STATE.cam_list = (camera *)malloc(API_STATE.device_list.nDeviceNum * sizeof(camera));

    unsigned int i;
    for (i = 0; i < API_STATE.device_list.nDeviceNum; i++)
    {
        if (!check_hik_err(&ret, MV_CC_CreateHandle(&API_STATE.cam_list[i].handle, API_STATE.device_list.pDeviceInfo[i])))
        {
            goto error;
        }

        if (!check_hik_err(&ret, MV_CC_OpenDevice(API_STATE.cam_list[i].handle, 0, 0)))
        {
            MV_CC_DestroyHandle(API_STATE.cam_list[i].handle);
            goto error;
        }

        if (!check_hik_err(&ret, MV_CC_SetImageNodeNum(API_STATE.cam_list[i].handle, 2)))
        {
            MV_CC_CloseDevice(API_STATE.cam_list[i].handle);
            MV_CC_DestroyHandle(API_STATE.cam_list[i].handle);
            goto error;
        }

        if (!check_hik_err(&ret, MV_CC_StartGrabbing(API_STATE.cam_list[i].handle)))
        {
            MV_CC_CloseDevice(API_STATE.cam_list[i].handle);
            MV_CC_DestroyHandle(API_STATE.cam_list[i].handle);
            goto error;
        }
    }
    *device_num = API_STATE.device_list.nDeviceNum;
    return ret;

error:
    for (unsigned int j = 0; j < i; ++j)
    {
        MV_CC_CloseDevice(API_STATE.cam_list[j].handle);
        MV_CC_DestroyHandle(API_STATE.cam_list[j].handle);
    }
    free(API_STATE.cam_list);
    API_STATE.cam_list = NULL;
    return ret;
}

api_error get_frame(unsigned int cam_idx, unsigned char *mem, unsigned int buffer_size)
{
    api_error ret = {false, MV_OK};
    if (!check_hik_err(&ret, MV_CC_GetImageBuffer(API_STATE.cam_list[cam_idx].handle, &API_STATE.cam_list[cam_idx].frame, 10)))
    {
        return ret;
    }

    MV_CC_PIXEL_CONVERT_PARAM_EX param = {0};
    MV_FRAME_OUT *frame = &API_STATE.cam_list[cam_idx].frame;
    param.nWidth = frame->stFrameInfo.nWidth;
    param.nHeight = frame->stFrameInfo.nHeight;
    param.pSrcData = frame->pBufAddr;
    param.nSrcDataLen = frame->stFrameInfo.nFrameLenEx;

    param.enSrcPixelType = frame->stFrameInfo.enPixelType;
    param.enDstPixelType = PixelType_Gvsp_RGB8_Packed;
    param.pDstBuffer = mem;
    param.nDstBufferSize = buffer_size;
    check_hik_err(&ret, MV_CC_ConvertPixelTypeEx(API_STATE.cam_list[cam_idx].handle, &param));

    check_hik_err(&ret, MV_CC_FreeImageBuffer(API_STATE.cam_list[cam_idx].handle, &API_STATE.cam_list[cam_idx].frame));
    return ret;
}

api_error set_float_param(unsigned int cam_idx, const char *param_name, float value)
{
    MVCC_FLOATVALUE stFloatValue = {0};
    MV_CC_GetFloatValue(API_STATE.cam_list[cam_idx].handle, param_name, &stFloatValue);
    printf("GetFloatValue: %f %f %f\n", stFloatValue.fCurValue, stFloatValue.fMax, stFloatValue.fMin);
    if (API_STATE.device_list.nDeviceNum <= cam_idx)
    {
        api_error ret = {true, CAMERA_API_INVALID_DEVICE_INDEX};
        return ret;
    }
    api_error ret = {false, MV_OK};
    check_hik_err(&ret, MV_CC_SetFloatValue(API_STATE.cam_list[cam_idx].handle, param_name, value));

    MV_CC_GetFloatValue(API_STATE.cam_list[cam_idx].handle, param_name, &stFloatValue);
    printf("GetFloatValue: %f %f %f\n", stFloatValue.fCurValue, stFloatValue.fMax, stFloatValue.fMin);
    return ret;
}

api_error set_int_param(unsigned int cam_idx, const char *param_name, unsigned int value)
{
    if (API_STATE.device_list.nDeviceNum <= cam_idx)
    {
        api_error ret = {true, CAMERA_API_INVALID_DEVICE_INDEX};
        return ret;
    }
    api_error ret = {false, MV_OK};
    check_hik_err(&ret, MV_CC_SetIntValue(API_STATE.cam_list[cam_idx].handle, param_name, value));
    return ret;
}

api_error set_enum_param(unsigned int cam_idx, const char *param_name, unsigned int value)
{
    if (API_STATE.device_list.nDeviceNum <= cam_idx)
    {
        api_error ret = {true, CAMERA_API_INVALID_DEVICE_INDEX};
        return ret;
    }
    api_error ret = {false, MV_OK};
    check_hik_err(&ret, MV_CC_SetEnumValue(API_STATE.cam_list[cam_idx].handle, param_name, value));
    return ret;
}