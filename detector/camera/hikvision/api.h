/**
 * @file api.h
 * @brief 为海康威视USB工业相机编写的相机控制API的函数声明
 *
 * @details
 * 该文件包含了相机控制相关的API定义，实现包含在同一目录下的lib.c文件中，包括初始化、关闭以及错误处理等
 * 若要更新lib.c中的函数，需要在此更新函数声明
 *
 * @author
 * 刘浩然
 *
 * @date
 * 2025-01-19
 *
 * @version
 * 1.0.0
 *
 * @copyright
 * Copyright (c) 2025, XMU RCS Robotics Lab. All rights reserved.
 *
 * @note
 * 暂无，过会写。
 */

#include "../camera_utils.h"

/**
 *  @brief  初始化API
 *  @return 成功，返回api_error{is_Hik_err=false, code=CAMERA_API_OK}；错误，返回api_error
 *  @remarks 一个程序只应该进行一次初始化，多次初始化会返回错误码CAMERA_API_ALREADY_INITIALIZED
 */
api_error init();

/**
 *  @brief  反初始化API
 *  @return 成功，返回api_error{is_Hik_err=false, code=CAMERA_API_OK}；错误，返回api_error
 *  @remarks 只能在已初始化API后调用，未初始化就进行反初始化会返回错误码CAMERA_API_NOT_INITIALIZED
 */
api_error final();

/**
 *  @brief  枚举设备，刷新设备列表
 *  @param  device_num  [OUT]    该无符号整型指针返回发现的工业相机的数量
 *  @return 成功，返回api_error{is_Hik_err=false, code=CAMERA_API_OK}；错误，返回api_error
 *  @remarks 设备列表的内存是在SDK内部存储的，每次调用该接口都会进行设备列表的刷新，过程中会进行设备列表内存的释放和申请，建议尽量避免多次枚举操作
 *  @remarks 本SDK仅仅考虑USB工业相机，若使用网口相机或其他类型工业相机，请修改此API，建议参考 /opt/MVS/doc 路径下的海康威视开发文档*/
api_error enumerate_devices(unsigned int *device_num);

/**
 * @brief  获取指定相机的图像帧
 * @param  cam_idx  [IN]    指定相机的索引
 * @param  mem      [OUT]   用于存储图像帧的内存指针，请根据具体相机型号设置
 * @return 成功，返回api_error{is_Hik_err=false, code=CAMERA_API_OK}；错误，返回api_error
 * @remarks 该函数会将获取到的图像帧存储在mem指向的内存中，mem的大小应该足够存储一帧图像，否则会发生越界访问
 */
api_error get_frame(unsigned int cam_idx, unsigned char *mem);

api_error set_enum_param(unsigned int cam_idx, const char *param_name, unsigned int value);

api_error set_int_param(unsigned int cam_idx, const char *param_name, unsigned int value);

api_error set_float_param(unsigned int cam_idx, const char *param_name, float value);
