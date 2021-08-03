/*************************************************************************
 * Copyright (c) 2015-2019, NVIDIA CORPORATION. All rights reserved.
 *
 * See LICENSE.txt for license information
 ************************************************************************/

#include <nccl.h>
#include <nccl_net.h>
#include <netinet/in.h>
#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

#include "bagua_net.h"

#define __hidden __attribute__((visibility("hidden")))

class BagueNet
{
public:
    static BagueNet &instance()
    {
        static BagueNet instance;
        return instance;
    }
    BagueNet(BagueNet const &) = delete;
    void operator=(BagueNet const &) = delete;

    int32_t devices(int32_t *ndev)
    {
        return bagua_net_c_devices(inner.get(), ndev);
    }

    int32_t get_properties(int32_t dev_id, NCCLNetPropertiesC *props)
    {
        return bagua_net_c_get_properties(inner.get(), dev_id, props);
    }

private:
    BagueNet()
    {
        inner = std::unique_ptr<BaguaNetC>(
            bagua_net_c_create(),
            [](BaguaNetC *ptr)
            {
                bagua_net_c_destroy(&ptr);
            });
    }
    BagueNet(BagueNet const &);
    void operator=(BagueNet const &);

private:
    std::unique_ptr<BaguaNetC> inner;
};

__hidden ncclResult_t baguaNetInit(ncclDebugLogger_t logFunction)
{
    BagueNet::instance();
    return ncclSuccess;
}

__hidden ncclResult_t baguaNetDevices(int *ndev)
{
    if (BagueNet::instance().devices(&ndev) != 0)
    {
        return ncclInternalError;
    }

    return ncclSuccess;
}

__hidden ncclResult_t baguaNetGetProperties(int dev, ncclNetProperties_v4_t *props)
{
    if (BagueNet::instance().get_properties(dev, (NCCLNetPropertiesC *)props) != 0)
    {
        return ncclInternalError;
    }
    return ncclInternalError;
}
__hidden ncclResult_t baguaNetListen(int dev, void *handle, void **listenComm) { return ncclInternalError; }
__hidden ncclResult_t baguaNetConnect(int dev, void *handle, void **sendComm) { return ncclInternalError; }
__hidden ncclResult_t baguaNetAccept(void *listenComm, void **recvComm) { return ncclInternalError; }
__hidden ncclResult_t baguaNetIsend(void *sendComm, void *data, int size, int type, void **request) { return ncclInternalError; }
__hidden ncclResult_t baguaNetIrecv(void *recvComm, void *data, int size, int type, void **request) { return ncclInternalError; }
__hidden ncclResult_t baguaNetFlush(void *recvComm, void *data, int size) { return ncclInternalError; }
__hidden ncclResult_t baguaNetTest(void *request, int *done, int *size) { return ncclInternalError; }
__hidden ncclResult_t baguaNetCloseSend(void *sendComm) { return ncclInternalError; }
__hidden ncclResult_t baguaNetCloseRecv(void *recvComm) { return ncclInternalError; }
__hidden ncclResult_t baguaNetCloseListen(void *listenComm) { return ncclInternalError; }

ncclNet_t NCCL_baguaNet_SYMBOL = {
    "BaguaNet",
    baguaNetInit,
    baguaNetDevices,
    baguaNetGetProperties,
    baguaNetListen,
    baguaNetConnect,
    baguaNetAccept,
    baguaNetIsend,
    baguaNetIrecv,
    baguaNetFlush,
    baguaNetTest,
    baguaNetCloseSend,
    baguaNetCloseRecv,
    baguaNetCloseListen};
