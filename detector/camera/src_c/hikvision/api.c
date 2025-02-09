/**
 * @file api.c
 * @brief 为海康威视USB工业相机编写的相机控制API的函数实现
 *
 * @details
 * 该文件包含了相机控制相关的API实现
 *
 * @date
 * 2025-01-19
 *
 * @copyright
 * Copyright (c) 2025, XMU RCS Robotics Lab. All rights reserved.
 *
 * @note
 * 本文件仅适用于海康威视USB工业相机，其他型号相机可能需要修改
 */

#include "MvErrorDefine.h"
#include "MvCameraControl.h"
#include "CameraParams.h"
#include <stdlib.h>
#include <string.h>
#include "../camera_utils.h"

typedef char bool;

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

inline static bool check_hik_err(APIError *ret, int err)
{
    if (err != MV_OK)
    {
        ret->is_thirdparty_err = true;
        ret->code = err;
        return false;
    }
    return true;
}

APIError read_int_param(unsigned int cam_idx, const char *param_name, unsigned int *value, unsigned int *min, unsigned int *max)
{
    if (API_STATE.device_list.nDeviceNum <= cam_idx)
    {
        APIError ret = {true, CAMERA_API_INVALID_DEVICE_INDEX};
        return ret;
    }
    APIError ret = {false, MV_OK};
    MVCC_INTVALUE_EX stIntValue = {0};
    check_hik_err(&ret, MV_CC_GetIntValueEx(API_STATE.cam_list[cam_idx].handle, param_name, &stIntValue));
    *value = stIntValue.nCurValue;
    *min = stIntValue.nMin;
    *max = stIntValue.nMax;
    return ret;
}

APIError read_enum_param(unsigned int cam_idx, const char *param_name, unsigned int *value)
{
    if (API_STATE.device_list.nDeviceNum <= cam_idx)
    {
        APIError ret = {true, CAMERA_API_INVALID_DEVICE_INDEX};
        return ret;
    }
    APIError ret = {false, MV_OK};
    MVCC_ENUMVALUE stEnumValue = {0};
    check_hik_err(&ret, MV_CC_GetEnumValue(API_STATE.cam_list[cam_idx].handle, param_name, &stEnumValue));
    *value = stEnumValue.nCurValue;
    return ret;
}

APIError read_float_param(unsigned int cam_idx, const char *param_name, float *value)
{
    if (API_STATE.device_list.nDeviceNum <= cam_idx)
    {
        APIError ret = {true, CAMERA_API_INVALID_DEVICE_INDEX};
        return ret;
    }
    APIError ret = {false, MV_OK};
    MVCC_FLOATVALUE stFloatValue = {0};
    check_hik_err(&ret, MV_CC_GetFloatValue(API_STATE.cam_list[cam_idx].handle, param_name, &stFloatValue));
    *value = stFloatValue.fCurValue;
    return ret;
}

APIError read_bool_param(unsigned int cam_idx, const char *param_name, bool *value)
{
    if (API_STATE.device_list.nDeviceNum <= cam_idx)
    {
        APIError ret = {true, CAMERA_API_INVALID_DEVICE_INDEX};
        return ret;
    }
    APIError ret = {false, MV_OK};
    MVCC_ENUMVALUE stEnumValue = {0};
    check_hik_err(&ret, MV_CC_GetEnumValue(API_STATE.cam_list[cam_idx].handle, param_name, &stEnumValue));
    *value = stEnumValue.nCurValue;
    return ret;
}

APIError set_float_param(unsigned int cam_idx, const char *param_name, float value)
{
    MVCC_FLOATVALUE stFloatValue = {0};
    MV_CC_GetFloatValue(API_STATE.cam_list[cam_idx].handle, param_name, &stFloatValue);
    printf("GetFloatValue: %f %f %f\n", stFloatValue.fCurValue, stFloatValue.fMax, stFloatValue.fMin);
    if (API_STATE.device_list.nDeviceNum <= cam_idx)
    {
        APIError ret = {true, CAMERA_API_INVALID_DEVICE_INDEX};
        return ret;
    }
    APIError ret = {false, MV_OK};
    check_hik_err(&ret, MV_CC_SetFloatValue(API_STATE.cam_list[cam_idx].handle, param_name, value));

    MV_CC_GetFloatValue(API_STATE.cam_list[cam_idx].handle, param_name, &stFloatValue);
    printf("GetFloatValue: %f %f %f\n", stFloatValue.fCurValue, stFloatValue.fMax, stFloatValue.fMin);
    return ret;
}

APIError set_int_param(unsigned int cam_idx, const char *param_name, unsigned int value)
{
    if (API_STATE.device_list.nDeviceNum <= cam_idx)
    {
        APIError ret = {true, CAMERA_API_INVALID_DEVICE_INDEX};
        return ret;
    }
    APIError ret = {false, MV_OK};
    check_hik_err(&ret, MV_CC_SetIntValue(API_STATE.cam_list[cam_idx].handle, param_name, value));
    return ret;
}

APIError set_enum_param(unsigned int cam_idx, const char *param_name, unsigned int value)
{
    if (API_STATE.device_list.nDeviceNum <= cam_idx)
    {
        APIError ret = {true, CAMERA_API_INVALID_DEVICE_INDEX};
        return ret;
    }
    APIError ret = {false, MV_OK};
    check_hik_err(&ret, MV_CC_SetEnumValue(API_STATE.cam_list[cam_idx].handle, param_name, value));
    return ret;
}

APIError set_bool_param(unsigned int cam_idx, const char *param_name, bool value)
{
    if (API_STATE.device_list.nDeviceNum <= cam_idx)
    {
        APIError ret = {true, CAMERA_API_INVALID_DEVICE_INDEX};
        return ret;
    }
    APIError ret = {false, MV_OK};
    check_hik_err(&ret, MV_CC_SetEnumValue(API_STATE.cam_list[cam_idx].handle, param_name, value));
    return ret;
}

APIError uninitialize_camera()
{
    APIError ret = {false, MV_OK};
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

APIError init()
{
    APIError ret = {false, MV_OK};
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

APIError final()
{
    if (!API_STATE.sdk_initialized)
    {
        APIError ret = {false, CAMERA_API_NOT_INITIALIZED};
        return ret;
    }

    APIError ret = uninitialize_camera();
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

APIError enumerate_devices(unsigned int *device_num)
{
    if (!API_STATE.sdk_initialized)
    {
        APIError ret = {false, CAMERA_API_NOT_INITIALIZED};
        return ret;
    }

    APIError ret = {false, MV_OK};
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

APIError get_frame(unsigned int cam_idx, unsigned char *mem, unsigned int buffer_size)
{
    APIError ret = {false, MV_OK};
    if (!check_hik_err(&ret, MV_CC_GetImageBuffer(API_STATE.cam_list[cam_idx].handle, &API_STATE.cam_list[cam_idx].frame, 1000)))
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
    param.enDstPixelType = PixelType_Gvsp_BGR8_Packed;
    param.pDstBuffer = mem;
    param.nDstBufferSize = buffer_size;
    check_hik_err(&ret, MV_CC_ConvertPixelTypeEx(API_STATE.cam_list[cam_idx].handle, &param));

    check_hik_err(&ret, MV_CC_FreeImageBuffer(API_STATE.cam_list[cam_idx].handle, &API_STATE.cam_list[cam_idx].frame));
    return ret;
}
