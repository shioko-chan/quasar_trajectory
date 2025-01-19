/**
 * @file lib.h
 * @brief 为海康威视USB工业相机编写的相机控制API
 *
 * @details
 * 该文件包含了相机控制相关的API定义，实现包含在同一目录下的lib.c文件中，包括初始化、关闭以及错误处理等
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

#ifndef HIK_API_LIB_H
#define HIK_API_LIB_H

#include "MvErrorDefine.h"
#include "MvCameraControl.h"
#include "CameraParams.h"
#include "lib.h"
#include <stdlib.h>
#include <string.h>

/**
 * @brief API错误码（自定义错误码）
 *
 * 定义了相机调用API的自定义错误码，用于描述API调用的结果状态。
 */
enum API_ERROR_CODE
{
  /**
   * @brief API调用成功
   *
   * 该错误码表示API调用成功，没有发生任何错误。
   */
  API_OK = 0,

  /**
   * @brief API已经初始化
   *
   * 该错误码表示API已完成初始化，不能重复初始化。
   */
  API_ALREADY_INITIALIZED = 1,

  /**
   * @brief API未初始化
   *
   * 该错误码表示API未完成初始化，需要先进行初始化。
   */
  API_NOT_INITIALIZED = 2,

};

/**
 * @brief API调用状态，
 * @param is_hik_err 该字段指示是否是海康威视的错误码
 * @param code 该字段指示错误码的值
 * @remarks 海康威视的错误码定义参考MvErrorDefine.h，自定义错误码参照枚举API_ERROR_CODE
 */
typedef struct
{
  bool is_hik_err;
  int code;
} api_error;

/**
 *  @brief  初始化API
 *  @return 成功，返回api_error{is_Hik_err=false, code=API_OK}；错误，返回api_error
 *  @remarks 一个程序只应该进行一次初始化，多次初始化会返回错误码API_ALREADY_INITIALIZED
 */
api_error init();

/**
 * *  @brief  反初始化API
 *  @return 成功，返回api_error{is_Hik_err=false, code=API_OK}；错误，返回api_error
 *  @remarks 只能在已初始化API后调用，未初始化就进行反初始化会返回错误码API_NOT_INITIALIZED
 */
api_error final();

/**
 *  @brief  枚举设备，刷新设备列表
 *
 *  @return 成功，返回api_error{is_Hik_err=false, code=API_OK}；错误，返回api_error
 *  @remarks 设备列表的内存是在SDK内部存储的，每次调用该接口都会进行设备列表的刷新，过程中会进行设备列表内存的释放和申请，建议尽量避免多次枚举操作
 *  @remarks 本SDK仅仅考虑USB工业相机，若使用网口相机或其他类型工业相机，请修改此API，建议参考 /opt/MVS/doc 路径下的海康威视开发文档*/
api_error enumerate_devices(unsigned int *device_num);

#endif