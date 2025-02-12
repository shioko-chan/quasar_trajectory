#ifndef CAMERA_UTILS_H
#define CAMERA_UTILS_H

#include <stdint.h>

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
#define CAMERA_API_MEM_OUT 5              // 该错误码表示内存分配失败。
#define CAMERA_API_NOT_WRITABLE 6         // 该错误码表示尝试写入不可写参数。
typedef char bool;

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

/**
 * @brief 枚举参数的字符串列表
 * @param current 当前枚举值的符号字符串
 * @param supported 支持的枚举项的符号字符串数组
 * @param count 支持的枚举项数量
 * @remarks 该结构体用于返回枚举参数的当前值和支持的枚举项
 */
typedef struct
{
    char *current;
    char **supported;
    unsigned int count;
} CEnumStringList;

/**
 * @brief 整型参数信息结构体
 * @param current 当前值
 * @param min 最小值
 * @param max 最大值
 * @param inc 增量（步长）
 * @remarks 该结构体用于描述整型参数的信息
 */
typedef struct
{
    int64_t current;
    int64_t min;
    int64_t max;
    int64_t inc;
} CIntParamInfo;

/**
 * @brief 浮点型参数信息结构体
 * @param current 当前值
 * @param min 最小值
 * @param max 最大值
 * @remarks 该结构体用于描述浮点型参数的信息
 */
typedef struct
{
    float current;
    float min;
    float max;
} CFloatParamInfo;

/**
 * @brief 字符串参数信息结构体
 * @param current 当前值
 * @param maxLength 最大长度
 * @remarks 该结构体用于描述字符串参数的信息
 */
typedef struct
{
    char *current;
    int64_t maxLength;
} CStringParamInfo;

#endif
