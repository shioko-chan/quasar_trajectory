#ifndef CAMERA_UTILS_H
#define CAMERA_UTILS_H

/**
 * @brief API错误码（自定义错误码）
 *
 * 定义了相机调用API的自定义错误码，用于描述API调用的结果状态。
 */
#define CAMERA_API_OK 0                   // 该错误码表示API调用成功，没有发生任何错误。
#define CAMERA_API_ALREADY_INITIALIZED 1  // 该错误码表示API已经初始化，不能重复初始化。
#define CAMERA_API_NOT_INITIALIZED 2      // 该错误码表示API未初始化，需要先进行初始化。
#define CAMERA_API_CAMERA_NOT_FOUND 3     // 该错误码表示没有找到相机。
#define CAMERA_API_INVALID_DEVICE_INDEX 4 // 该错误码表示设备索引无效。

/**
 * @brief API调用状态，
 * @param is_thirdparty_err 该字段指示是否是来自第三方相机SDK的错误码
 * @param code 该字段指示错误码的值
 * @remarks 海康威视的错误码定义参考MvErrorDefine.h，自定义错误码参照上述宏定义
 */
typedef struct
{
    char is_thirdparty_err;
    int code;
} APIError;

#endif
