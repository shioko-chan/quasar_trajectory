#include "lib.h"

/**
 * @brief API全局状态结构体
 * @param sdk_initialized 该字段指示是否已经完成SDK的初始化
 * @param device_enumerated 该字段指示是否已经完成设备枚举
 * @param device_list 该字段存储设备枚举列表
 */
static struct
{
    bool sdk_initialized;

    void **cam_handle_list;
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
    if (API_STATE.device_list.nDeviceNum > 0)
    {
        for (int i = 0; i < API_STATE.device_list.nDeviceNum; i++)
        {
            check_hik_err(&ret, MV_CC_CloseDevice(API_STATE.cam_handle_list[i]));
            check_hik_err(&ret, MV_CC_DestroyHandle(API_STATE.cam_handle_list[i]));
        }
        free(API_STATE.cam_handle_list);
    }
    memset(&API_STATE.device_list, 0, sizeof(MV_CC_DEVICE_INFO_LIST));
    API_STATE.cam_handle_list = NULL;
    return ret;
}

api_error init()
{
    api_error ret = {false, MV_OK};
    if (API_STATE.sdk_initialized)
    {
        ret.code = API_ALREADY_INITIALIZED;
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
        api_error ret = {false, API_NOT_INITIALIZED};
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
        api_error ret = {false, API_NOT_INITIALIZED};
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
    API_STATE.cam_handle_list = (void **)malloc(API_STATE.device_list.nDeviceNum * sizeof(void *));
    for (unsigned int i = 0; i < API_STATE.device_list.nDeviceNum; i++)
    {

        if (check_hik_err(&ret, MV_CC_CreateHandle(API_STATE.cam_handle_list + i, API_STATE.device_list.pDeviceInfo[i])))
        {
            if (check_hik_err(&ret, MV_CC_OpenDevice(API_STATE.cam_handle_list[i], 0, 0)))
            {
                continue;
            }
            MV_CC_DestroyHandle(API_STATE.cam_handle_list[i]);
        }
        for (unsigned int j = 0; j < i; ++j)
        {
            MV_CC_CloseDevice(API_STATE.cam_handle_list[j]);
            MV_CC_DestroyHandle(API_STATE.cam_handle_list[j]);
        }
        return ret;
    }
    *device_num = API_STATE.device_list.nDeviceNum;
    return ret;
}
