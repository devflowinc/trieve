# trieve_py_client.UserApi

All URIs are relative to *https://api.trieve.ai*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_user_api_key**](UserApi.md#delete_user_api_key) | **DELETE** /api/user/api_key/{api_key_id} | Delete User Api Key
[**get_user_api_keys**](UserApi.md#get_user_api_keys) | **GET** /api/user/api_key | Get User Api Keys
[**update_user**](UserApi.md#update_user) | **PUT** /api/user | Update User Org Role


# **delete_user_api_key**
> delete_user_api_key(api_key_id)

Delete User Api Key

Delete an api key for the auth'ed user.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to https://api.trieve.ai
# See configuration.py for a list of all supported configuration parameters.
configuration = trieve_py_client.Configuration(
    host = "https://api.trieve.ai"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: ApiKey
configuration.api_key['ApiKey'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['ApiKey'] = 'Bearer'

# Enter a context with an instance of the API client
with trieve_py_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = trieve_py_client.UserApi(api_client)
    api_key_id = 'api_key_id_example' # str | The id of the api key to delete

    try:
        # Delete User Api Key
        api_instance.delete_user_api_key(api_key_id)
    except Exception as e:
        print("Exception when calling UserApi->delete_user_api_key: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **api_key_id** | **str**| The id of the api key to delete | 

### Return type

void (empty response body)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**204** | Confirmation that the api key was deleted |  -  |
**400** | Service error relating to creating api_key for the user |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_user_api_keys**
> List[ApiKeyRespBody] get_user_api_keys()

Get User Api Keys

Get the api keys which belong to the auth'ed user. The actual api key values are not returned, only the ids, names, and creation dates.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.api_key_resp_body import ApiKeyRespBody
from trieve_py_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to https://api.trieve.ai
# See configuration.py for a list of all supported configuration parameters.
configuration = trieve_py_client.Configuration(
    host = "https://api.trieve.ai"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: ApiKey
configuration.api_key['ApiKey'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['ApiKey'] = 'Bearer'

# Enter a context with an instance of the API client
with trieve_py_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = trieve_py_client.UserApi(api_client)

    try:
        # Get User Api Keys
        api_response = api_instance.get_user_api_keys()
        print("The response of UserApi->get_user_api_keys:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling UserApi->get_user_api_keys: %s\n" % e)
```



### Parameters

This endpoint does not need any parameter.

### Return type

[**List[ApiKeyRespBody]**](ApiKeyRespBody.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | JSON body representing the api_key for the user |  -  |
**400** | Service error relating to creating api_key for the user |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_user**
> update_user(tr_organization, update_user_org_role_req_payload)

Update User Org Role

Update a user's information for the org specified via header. If the user_id is not provided, the auth'ed user will be updated. If the user_id is provided, the role of the auth'ed user or api key must be an admin (1) or owner (2) of the organization.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.update_user_org_role_req_payload import UpdateUserOrgRoleReqPayload
from trieve_py_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to https://api.trieve.ai
# See configuration.py for a list of all supported configuration parameters.
configuration = trieve_py_client.Configuration(
    host = "https://api.trieve.ai"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: ApiKey
configuration.api_key['ApiKey'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['ApiKey'] = 'Bearer'

# Enter a context with an instance of the API client
with trieve_py_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = trieve_py_client.UserApi(api_client)
    tr_organization = 'tr_organization_example' # str | The organization id to use for the request
    update_user_org_role_req_payload = trieve_py_client.UpdateUserOrgRoleReqPayload() # UpdateUserOrgRoleReqPayload | JSON request payload to update user information for the auth'ed user

    try:
        # Update User Org Role
        api_instance.update_user(tr_organization, update_user_org_role_req_payload)
    except Exception as e:
        print("Exception when calling UserApi->update_user: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tr_organization** | **str**| The organization id to use for the request | 
 **update_user_org_role_req_payload** | [**UpdateUserOrgRoleReqPayload**](UpdateUserOrgRoleReqPayload.md)| JSON request payload to update user information for the auth&#39;ed user | 

### Return type

void (empty response body)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**204** | Confirmation that the user&#39;s role was updated |  -  |
**400** | Service error relating to updating the user |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

