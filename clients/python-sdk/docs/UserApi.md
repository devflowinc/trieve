# trieve_py_client.UserApi

All URIs are relative to *https://api.trieve.ai*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_user_api_key**](UserApi.md#delete_user_api_key) | **DELETE** /api/user/api_key/{api_key_id} | Delete User Api Key
[**set_user_api_key**](UserApi.md#set_user_api_key) | **POST** /api/user/api_key | Set User Api Key
[**update_user**](UserApi.md#update_user) | **PUT** /api/user | Update User


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

# **set_user_api_key**
> SetUserApiKeyResponse set_user_api_key(set_user_api_key_request)

Set User Api Key

Create a new api key for the auth'ed user. Successful response will contain the newly created api key. If a write role is assigned the api key will have permission level of the auth'ed user who calls this endpoint.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.set_user_api_key_request import SetUserApiKeyRequest
from trieve_py_client.models.set_user_api_key_response import SetUserApiKeyResponse
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
    set_user_api_key_request = trieve_py_client.SetUserApiKeyRequest() # SetUserApiKeyRequest | JSON request payload to create a new user api key

    try:
        # Set User Api Key
        api_response = api_instance.set_user_api_key(set_user_api_key_request)
        print("The response of UserApi->set_user_api_key:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling UserApi->set_user_api_key: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **set_user_api_key_request** | [**SetUserApiKeyRequest**](SetUserApiKeyRequest.md)| JSON request payload to create a new user api key | 

### Return type

[**SetUserApiKeyResponse**](SetUserApiKeyResponse.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | JSON body representing the api_key for the user |  -  |
**400** | Service error relating to creating api_key for the user |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_user**
> update_user(update_user_org_role_data)

Update User

Update a user's information. If the user_id is not provided, the auth'ed user will be updated. If the user_id is provided, the role of the auth'ed user or api key must be an admin (1) or owner (2) of the organization.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.update_user_org_role_data import UpdateUserOrgRoleData
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
    update_user_org_role_data = trieve_py_client.UpdateUserOrgRoleData() # UpdateUserOrgRoleData | JSON request payload to update user information for the auth'ed user

    try:
        # Update User
        api_instance.update_user(update_user_org_role_data)
    except Exception as e:
        print("Exception when calling UserApi->update_user: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **update_user_org_role_data** | [**UpdateUserOrgRoleData**](UpdateUserOrgRoleData.md)| JSON request payload to update user information for the auth&#39;ed user | 

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

