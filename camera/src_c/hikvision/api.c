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
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../camera_utils.h"
#include "api.h"

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

void free_mem(void *ptr)
{
    free(ptr);
}

APIError get_int_param(unsigned int cam_idx, const char *param_name, CIntParamInfo *out_info)
{
    if (API_STATE.device_list.nDeviceNum <= cam_idx)
    {
        APIError ret = {true, CAMERA_API_INVALID_DEVICE_INDEX};
        return ret;
    }

    APIError api_error = {false, MV_OK};

    void *handle = API_STATE.cam_list[cam_idx].handle;

    MVCC_INTVALUE_EX stIntValue = {0};
    int nRet = MV_CC_GetIntValueEx(handle, param_name, &stIntValue);
    if (!check_hik_err(&api_error, nRet))
    {
        return api_error;
    }

    // 填充返回结构体信息
    out_info->current = stIntValue.nCurValue;
    out_info->min = stIntValue.nMin;
    out_info->max = stIntValue.nMax;
    out_info->inc = stIntValue.nInc;

    return api_error;
}

APIError get_float_param(unsigned int cam_idx, const char *param_name, CFloatParamInfo *out_info)
{
    if (API_STATE.device_list.nDeviceNum <= cam_idx)
    {
        APIError ret = {true, CAMERA_API_INVALID_DEVICE_INDEX};
        return ret;
    }

    APIError api_error = {false, MV_OK};

    void *handle = API_STATE.cam_list[cam_idx].handle;

    MVCC_FLOATVALUE stFloatValue = {0};
    int nRet = MV_CC_GetFloatValue(handle, param_name, &stFloatValue);
    if (!check_hik_err(&api_error, nRet))
    {
        return api_error;
    }

    // 填充返回结构体信息
    out_info->current = stFloatValue.fCurValue;
    out_info->min = stFloatValue.fMin;
    out_info->max = stFloatValue.fMax;

    return api_error;
}

APIError get_bool_param(unsigned int cam_idx, const char *param_name, bool *out_info)
{
    if (API_STATE.device_list.nDeviceNum <= cam_idx)
    {
        APIError ret = {true, CAMERA_API_INVALID_DEVICE_INDEX};
        return ret;
    }

    APIError api_error = {false, MV_OK};

    void *handle = API_STATE.cam_list[cam_idx].handle;

    int nRet = MV_CC_GetBoolValue(handle, param_name, out_info);
    check_hik_err(&api_error, nRet);

    return api_error;
}

APIError get_string_param(unsigned int cam_idx, const char *param_name, CStringParamInfo *out_info)
{
    if (API_STATE.device_list.nDeviceNum <= cam_idx)
    {
        APIError ret = {true, CAMERA_API_INVALID_DEVICE_INDEX};
        return ret;
    }

    APIError api_error = {false, MV_OK};

    void *handle = API_STATE.cam_list[cam_idx].handle;

    MVCC_STRINGVALUE stStringValue = {0};
    int nRet = MV_CC_GetStringValue(handle, param_name, &stStringValue);
    if (!check_hik_err(&api_error, nRet))
    {
        return api_error;
    }

    // 填充返回结构体信息
    out_info->current = strdup(stStringValue.chCurValue);
    if (out_info->current == NULL)
    {
        api_error.is_thirdparty_err = false;
        api_error.code = CAMERA_API_MEM_OUT;
        return api_error;
    }
    out_info->current = stStringValue.chCurValue;
    out_info->maxLength = stStringValue.nMaxLength;

    return api_error;
}

APIError get_enum_param(unsigned int cam_idx, const char *param_name, CEnumStringList *out_list)
{
    if (API_STATE.device_list.nDeviceNum <= cam_idx)
    {
        APIError ret = {true, CAMERA_API_INVALID_DEVICE_INDEX};
        return ret;
    }

    APIError api_error = {false, MV_OK};

    void *handle = API_STATE.cam_list[cam_idx].handle;

    // 获取枚举值信息
    MVCC_ENUMVALUE stEnumValue = {0};
    int nRet = MV_CC_GetEnumValue(handle, param_name, &stEnumValue);
    if (!check_hik_err(&api_error, nRet))
    {
        return api_error;
    }

    // 获取当前枚举值的符号字符串
    MVCC_ENUMENTRY stEnumEntry = {0};
    stEnumEntry.nValue = stEnumValue.nCurValue;
    nRet = MV_CC_GetEnumEntrySymbolic(handle, param_name, &stEnumEntry);
    if (!check_hik_err(&api_error, nRet))
    {
        return api_error;
    }

    // 为当前枚举值分配内存并复制字符串
    out_list->current = strdup(stEnumEntry.chSymbolic);
    if (out_list->current == NULL)
    {
        api_error.is_thirdparty_err = false;
        api_error.code = CAMERA_API_MEM_OUT;
        return api_error;
    }

    // 分配字符串数组，用于保存支持的枚举项
    out_list->count = stEnumValue.nSupportedNum;
    if (out_list->count > 0)
    {
        out_list->supported = (char **)malloc(out_list->count * sizeof(char *));
        if (out_list->supported == NULL)
        {
            free(out_list->current);
            api_error.is_thirdparty_err = false;
            api_error.code = CAMERA_API_MEM_OUT;
            return api_error;
        }
    }
    else
    {
        out_list->supported = NULL;
    }

    // 循环读取每个支持枚举值的符号
    for (unsigned int i = 0; i < out_list->count; i++)
    {
        stEnumEntry.nValue = stEnumValue.nSupportValue[i];
        nRet = MV_CC_GetEnumEntrySymbolic(handle, param_name, &stEnumEntry);
        if (!check_hik_err(&api_error, nRet))
        {
            // 释放已分配的内存
            for (unsigned int j = 0; j < i; j++)
            {
                free(out_list->supported[j]);
            }
            free(out_list->supported);
            free(out_list->current);
            return api_error;
        }
        out_list->supported[i] = strdup(stEnumEntry.chSymbolic);
        if (out_list->supported[i] == NULL)
        {
            // 内存分配失败，释放所有已分配的内存
            for (unsigned int j = 0; j < i; j++)
            {
                free(out_list->supported[j]);
            }
            free(out_list->supported);
            free(out_list->current);
            api_error.is_thirdparty_err = false;
            api_error.code = CAMERA_API_MEM_OUT;
            return api_error;
        }
    }

    return api_error;
}

APIError set_int_param(unsigned int cam_idx, const char *param_name, unsigned int value)
{
    if (API_STATE.device_list.nDeviceNum <= cam_idx)
    {
        APIError ret = {true, CAMERA_API_INVALID_DEVICE_INDEX};
        return ret;
    }

    APIError ret = {false, MV_OK};

    void *handle = API_STATE.cam_list[cam_idx].handle;

    // 读取当前整数值用于调试
    MVCC_INTVALUE_EX stIntValue = {0};
    int nRet = MV_CC_GetIntValueEx(handle, param_name, &stIntValue);
    if (!check_hik_err(&ret, nRet))
    {
        printf("Get %s Fail! nRet [0x%x]\n", param_name, nRet);
        return ret;
    }
    printf("Get %s = [%ld] Success! (Range: [%ld, %ld])\n",
           param_name, stIntValue.nCurValue, stIntValue.nMin, stIntValue.nMax);

    // 检查访问权限
    enum MV_XML_AccessMode enAccessMode = AM_NI;
    nRet = MV_XML_GetNodeAccessMode(handle, param_name, &enAccessMode);
    if (MV_OK == nRet && AM_RW == enAccessMode)
    {
        nRet = MV_CC_SetIntValueEx(handle, param_name, value);
        if (!check_hik_err(&ret, nRet))
        {
            printf("Set %s Fail! nRet [0x%x]\n", param_name, nRet);
            return ret;
        }
        else
        {
            printf("Set %s = [%u] Success!\n", param_name, value);
        }
    }
    else
    {
        printf("Parameter %s is not writable.\n", param_name);
    }
    return ret;
}

APIError set_float_param(unsigned int cam_idx, const char *param_name, float value)
{
    if (API_STATE.device_list.nDeviceNum <= cam_idx)
    {
        APIError ret = {true, CAMERA_API_INVALID_DEVICE_INDEX};
        return ret;
    }

    APIError ret = {false, MV_OK};

    void *handle = API_STATE.cam_list[cam_idx].handle;

    // 读取当前浮点值（用于打印调试）
    MVCC_FLOATVALUE stFloatValue = {0};
    int nRet = MV_CC_GetFloatValue(handle, param_name, &stFloatValue);
    if (!check_hik_err(&ret, nRet))
    {
        printf("Get %s Fail! nRet [0x%x]\n", param_name, nRet);
        return ret;
    }
    printf("Get %s = [%f] Success! (Range: [%f, %f])\n",
           param_name, stFloatValue.fCurValue, stFloatValue.fMin, stFloatValue.fMax);

    // 查询节点访问权限
    enum MV_XML_AccessMode enAccessMode = AM_NI;
    nRet = MV_XML_GetNodeAccessMode(handle, param_name, &enAccessMode);
    if (MV_OK == nRet && AM_RW == enAccessMode)
    {
        // 设置浮点参数为指定 value
        nRet = MV_CC_SetFloatValue(handle, param_name, value);
        if (!check_hik_err(&ret, nRet))
        {
            printf("Set %s Fail! nRet [0x%x]\n", param_name, nRet);
            return ret;
        }
        else
        {
            // 可再次读取确认
            nRet = MV_CC_GetFloatValue(handle, param_name, &stFloatValue);
            if (nRet == MV_OK)
            {
                printf("Set %s = [%f] Success!\n", param_name, stFloatValue.fCurValue);
            }
        }
    }
    else
    {
        printf("Parameter %s is not writable.\n", param_name);
    }

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

    void *handle = API_STATE.cam_list[cam_idx].handle;

    // 读取当前 bool 值用于调试
    bool bValue = false;
    int nRet = MV_CC_GetBoolValue(handle, param_name, &bValue);
    if (!check_hik_err(&ret, nRet))
    {
        printf("Get %s Fail! nRet [0x%x]\n", param_name, nRet);
        return ret;
    }
    printf("Get %s = [%d] Success!\n", param_name, bValue);

    // 检查访问权限
    enum MV_XML_AccessMode enAccessMode = AM_NI;
    nRet = MV_XML_GetNodeAccessMode(handle, param_name, &enAccessMode);
    if (MV_OK == nRet && AM_RW == enAccessMode)
    {
        nRet = MV_CC_SetBoolValue(handle, param_name, value);
        if (!check_hik_err(&ret, nRet))
        {
            printf("Set %s Fail! nRet [0x%x]\n", param_name, nRet);
            return ret;
        }
        else
        {
            printf("Set %s = [%d] Success!\n", param_name, value);
        }
    }
    else
    {
        printf("Parameter %s is not writable.\n", param_name);
    }
    return ret;
}

APIError set_string_param(unsigned int cam_idx, const char *param_name, const char *value)
{
    APIError ret = {false, MV_OK};

    // 检查设备索引是否有效
    if (API_STATE.device_list.nDeviceNum <= cam_idx)
    {
        ret.is_thirdparty_err = true;
        ret.code = CAMERA_API_INVALID_DEVICE_INDEX;
        return ret;
    }
    void *handle = API_STATE.cam_list[cam_idx].handle;

    // 读取当前字符串值（用于调试）
    MVCC_STRINGVALUE stStringValue;
    memset(&stStringValue, 0, sizeof(MVCC_STRINGVALUE));
    int nRet = MV_CC_GetStringValue(handle, param_name, &stStringValue);
    if (!check_hik_err(&ret, nRet))
    {
        printf("Get %s Fail! nRet [0x%x]\n", param_name, nRet);
        return ret;
    }
    else
    {
        printf("Get %s = [%s] Success!\n", param_name, stStringValue.chCurValue);
    }

    // 检查节点访问模式，确认该参数可写
    enum MV_XML_AccessMode enAccessMode = AM_NI;
    nRet = MV_XML_GetNodeAccessMode(handle, param_name, &enAccessMode);
    if (MV_OK == nRet && AM_RW == enAccessMode)
    {
        nRet = MV_CC_SetStringValue(handle, param_name, value);
        if (!check_hik_err(&ret, nRet))
        {
            printf("Set %s Fail! nRet [0x%x]\n", param_name, nRet);
            return ret;
        }
        else
        {
            printf("Set %s = [%s] Success!\n", param_name, stStringValue.chCurValue);
        }
    }
    else
    {
        printf("Parameter %s is not writable.\n", param_name);
    }
    return ret;
}

APIError set_enum_param(unsigned int cam_idx, const char *param_name, const char *value)
{
    APIError ret = {false, MV_OK};

    // 检查设备索引是否有效
    if (API_STATE.device_list.nDeviceNum <= cam_idx)
    {
        ret.is_thirdparty_err = true;
        ret.code = CAMERA_API_INVALID_DEVICE_INDEX;
        return ret;
    }
    void *handle = API_STATE.cam_list[cam_idx].handle;

    // 检查节点访问模式
    enum MV_XML_AccessMode enAccessMode = AM_NI;
    int nRet = MV_XML_GetNodeAccessMode(handle, param_name, &enAccessMode);
    if (!check_hik_err(&ret, nRet))
    {
        printf("Get Node Access Mode for %s Fail! nRet [0x%x]\n", param_name, nRet);
        return ret;
    }
    if (enAccessMode != AM_RW)
    {
        printf("Parameter %s is not writable (Access mode is not RW)\n", param_name);
        ret.code = CAMERA_API_NOT_WRITABLE;
        return ret;
    }

    // 调用 SDK API 设置枚举参数值
    nRet = MV_CC_SetEnumValueByString(handle, param_name, value);
    if (!check_hik_err(&ret, nRet))
    {
        printf("Set %s to %s Fail! nRet [0x%x]\n", param_name, value, nRet);
        return ret;
    }
    else
    {
        printf("Set %s to %s Success!\n", param_name, value);
    }

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
