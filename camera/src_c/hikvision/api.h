/**
 * @file api.h
 * @brief 为海康威视USB工业相机编写的相机控制API的函数声明
 *
 * @details
 * 该文件包含了相机控制相关的API定义，实现包含在同一目录下的lib.c文件中，包括初始化、关闭以及错误处理、参数设置等功能
 * 若要更新lib.c中的函数，需要在此更新函数声明
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

#include "../camera_utils.h"

/**
 * @brief 释放内存
 * @param ptr [IN] 需要释放的内存指针
 */
void free_mem(void *ptr);

/**
 *  @brief  初始化API
 *  @return 成功，返回APIError{is_Hik_err=false, code=CAMERA_API_OK}；错误，返回APIError
 *  @remarks 一个程序只应该进行一次初始化，多次初始化会返回错误码CAMERA_API_ALREADY_INITIALIZED
 */
APIError init();

/**
 *  @brief  反初始化API
 *  @return 成功，返回APIError{is_Hik_err=false, code=CAMERA_API_OK}；错误，返回APIError
 *  @remarks 只能在已初始化API后调用，未初始化就进行反初始化会返回错误码CAMERA_API_NOT_INITIALIZED
 */
APIError final();

/**
 *  @brief  枚举设备，刷新设备列表
 *  @param  device_num  [OUT]    该无符号整型指针返回发现的工业相机的数量
 *  @return 成功，返回APIError{is_Hik_err=false, code=CAMERA_API_OK}；错误，返回APIError
 *  @remarks 设备列表的内存是在SDK内部存储的，每次调用该接口都会进行设备列表的刷新，过程中会进行设备列表内存的释放和申请，建议尽量避免多次枚举操作
 *  @remarks 本SDK仅仅考虑USB工业相机，若使用网口相机或其他类型工业相机，请修改此API，建议参考 /opt/MVS/doc 路径下的海康威视开发文档*/
APIError enumerate_devices(unsigned int *device_num);

/**
 * @brief  获取指定相机的图像帧
 * @param  cam_idx  [IN]    指定相机的索引
 * @param  mem      [OUT]   用于存储图像帧的内存指针，请根据具体相机型号设置
 * @return 成功，返回APIError{is_Hik_err=false, code=CAMERA_API_OK}；错误，返回APIError
 * @remarks 该函数会将获取到的图像帧存储在mem指向的内存中，mem的大小应该足够存储一帧图像，否则会发生越界访问
 */
APIError get_frame(unsigned int cam_idx, unsigned char *mem, unsigned int buffer_size);

/**
 * @brief 设置相机枚举参数
 * @param cam_idx [IN] 指定相机的索引
 * @param param_name [IN] 参数名称
 * @param value [IN] 参数值
 * @return 成功，返回APIError{is_Hik_err=false, code=CAMERA_API_OK}；错误，返回APIError
 * @remarks 该函数用于设置相机的枚举类型参数，param_name为参数名称，value为参数值
 * @remarks 具体参数列表参见海康威视相机手册
 */
APIError set_enum_param(unsigned int cam_idx, const char *param_name, const char *value);

/**
 * @brief 设置相机整型参数
 * @param cam_idx [IN] 指定相机的索引
 * @param param_name [IN] 参数名称
 * @param value [IN] 参数值
 * @return 成功，返回APIError{is_Hik_err=false, code=CAMERA_API_OK}；错误，返回APIError
 * @remarks 该函数用于设置相机的整型参数，param_name为参数名称，value为参数值
 * @remarks 具体参数列表参见海康威视相机手册
 */
APIError set_int_param(unsigned int cam_idx, const char *param_name, unsigned int value);

/**
 * @brief 设置相机浮点型参数
 * @param cam_idx [IN] 指定相机的索引
 * @param param_name [IN] 参数名称
 * @param value [IN] 参数值
 * @return 成功，返回APIError{is_Hik_err=false, code=CAMERA_API_OK}；错误，返回APIError
 * @remarks 该函数用于设置相机的浮点型参数，param_name为参数名称，value为参数值
 * @remarks 具体参数列表参见海康威视相机手册
 */
APIError set_float_param(unsigned int cam_idx, const char *param_name, float value);

/**
 * @brief 设置相机布尔型参数
 * @param cam_idx [IN] 指定相机的索引
 * @param param_name [IN] 参数名称
 * @param value [IN] 参数值
 * @return 成功，返回APIError{is_Hik_err=false, code=CAMERA_API_OK}；错误，返回APIError
 * @remarks 该函数用于设置相机的布尔型参数，param_name为参数名称，value为参数值
 * @remarks 具体参数列表参见海康威视相机手册
 */
APIError set_bool_param(unsigned int cam_idx, const char *param_name, bool value);

/**
 * @brief 设置相机字符串型参数
 * @param cam_idx [IN] 指定相机的索引
 * @param param_name [IN] 参数名称
 * @param value [IN] 参数值
 * @return 成功，返回APIError{is_Hik_err=false, code=CAMERA_API_OK}；错误，返回APIError
 * @remarks 该函数用于设置相机的字符串参数，param_name为参数名称，value为参数值
 * @remarks 具体参数列表参见海康威视相机手册
 */
APIError set_string_param(unsigned int cam_idx, const char *param_name, const char *value);

/**
 * @brief 读取指定相机指定整数参数的当前值和范围信息，并填充到 out_info 中。
 *
 * @param cam_idx  [IN] 相机在全局 cam_list 中的索引，从零开始
 * @param param_name [IN] 参数名称（字符串）
 * @param out_info [OUT] 指向返回结果结构体的指针，函数成功后会填充当前值、最小值、最大值和增量
 * @return APIError 错误信息，如果成功则 code == MV_OK
 * @remarks 具体参数列表参见海康威视相机手册
 */
APIError get_int_param(unsigned int cam_idx, const char *param_name, CIntParamInfo *out_info);

/**
 * @brief 读取指定相机指定浮点数参数的当前值和范围信息，并填充到 out_info 中。
 *
 * @param cam_idx  [IN] 相机在全局 cam_list 中的索引，从零开始
 * @param param_name [IN] 参数名称（字符串）
 * @param out_info [OUT] 指向返回结果结构体的指针，函数成功后会填充当前值、最小值、最大值
 * @return APIError 错误信息，如果成功则 code == MV_OK
 * @remarks 具体参数列表参见海康威视相机手册
 */
APIError get_float_param(unsigned int cam_idx, const char *param_name, CFloatParamInfo *out_info);

/**
 * @brief 读取相机布尔型参数
 * @param cam_idx [IN] 指定相机的索引，从零开始
 * @param param_name [IN] 参数名称
 * @param out_info [OUT] 参数值
 * @return 成功，返回APIError{is_Hik_err=false, code=CAMERA_API_OK}；错误，返回APIError
 * @remarks 具体参数列表参见海康威视相机手册
 */
APIError get_bool_param(unsigned int cam_idx, const char *param_name, bool *out_info);

/**
 * @brief 读取指定相机指定字符串参数的当前值和该位置能接受字符串参数的最大长度
 * @param cam_idx [IN] 相机在全局 cam_list 中的索引
 * @param param_name [IN] 字符串参数的名称（字符串）
 * @param out_info [OUT] 用于返回字符串参数信息，调用者负责释放其中分配的内存
 * @return APIError 错误信息，如果成功则 code == MV_OK
 * @remarks 具体参数列表参见海康威视相机手册
 */
APIError get_string_param(unsigned int cam_idx, const char *param_name, CStringParamInfo *out_info);
/**
 * @brief 读取指定相机指定枚举参数的当前值和支持的枚举项，
 *        将结果以 CEnumStringList 结构体形式返回。
 * @param cam_idx  相机在全局 cam_list 中的索引
 * @param param_name 枚举参数的名称（字符串）
 * @param out_list 用于返回枚举参数字符串列表，调用者负责释放其中分配的内存
 * @return APIError 错误信息，如果成功则 code == MV_OK
 * @remarks 具体参数列表参见海康威视相机手册
 */
APIError get_enum_param(unsigned int cam_idx, const char *param_name, CEnumStringList *out_list);
