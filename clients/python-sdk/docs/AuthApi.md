# trieve_py_client.AuthApi

All URIs are relative to *https://api.trieve.ai*

Method | HTTP request | Description
------------- | ------------- | -------------
[**callback**](AuthApi.md#callback) | **GET** /api/auth/callback | OpenID Connect callback
[**get_me**](AuthApi.md#get_me) | **GET** /api/auth/me | Get Me
[**login**](AuthApi.md#login) | **GET** /api/auth | Login
[**logout**](AuthApi.md#logout) | **DELETE** /api/auth | Logout


# **callback**
> SlimUser callback()

OpenID Connect callback

This is the callback route for the OAuth provider, it should not be called directly. Redirects to browser with set-cookie header.

### Example


```python
import trieve_py_client
from trieve_py_client.models.slim_user import SlimUser
from trieve_py_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to https://api.trieve.ai
# See configuration.py for a list of all supported configuration parameters.
configuration = trieve_py_client.Configuration(
    host = "https://api.trieve.ai"
)


# Enter a context with an instance of the API client
with trieve_py_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = trieve_py_client.AuthApi(api_client)

    try:
        # OpenID Connect callback
        api_response = api_instance.callback()
        print("The response of AuthApi->callback:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthApi->callback: %s\n" % e)
```



### Parameters

This endpoint does not need any parameter.

### Return type

[**SlimUser**](SlimUser.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Response that returns with set-cookie header |  -  |
**400** | Email or password empty or incorrect |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_me**
> SlimUser get_me()

Get Me

Get the user corresponding to your current auth credentials.

### Example

* Api Key Authentication (ApiKey):

```python
import trieve_py_client
from trieve_py_client.models.slim_user import SlimUser
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
    api_instance = trieve_py_client.AuthApi(api_client)

    try:
        # Get Me
        api_response = api_instance.get_me()
        print("The response of AuthApi->get_me:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthApi->get_me: %s\n" % e)
```



### Parameters

This endpoint does not need any parameter.

### Return type

[**SlimUser**](SlimUser.md)

### Authorization

[ApiKey](../README.md#ApiKey)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | The user corresponding to your current auth credentials |  -  |
**400** | Error message indicitating you are not currently signed in |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **login**
> login(organization_id=organization_id, redirect_uri=redirect_uri, inv_code=inv_code)

Login

This will redirect you to the OAuth provider for authentication with email/pass, SSO, Google, Github, etc.

### Example


```python
import trieve_py_client
from trieve_py_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to https://api.trieve.ai
# See configuration.py for a list of all supported configuration parameters.
configuration = trieve_py_client.Configuration(
    host = "https://api.trieve.ai"
)


# Enter a context with an instance of the API client
with trieve_py_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = trieve_py_client.AuthApi(api_client)
    organization_id = 'organization_id_example' # str | ID of organization to authenticate into (optional)
    redirect_uri = 'redirect_uri_example' # str | URL to redirect to after successful login (optional)
    inv_code = 'inv_code_example' # str | Code sent via email as a result of successful call to send_invitation (optional)

    try:
        # Login
        api_instance.login(organization_id=organization_id, redirect_uri=redirect_uri, inv_code=inv_code)
    except Exception as e:
        print("Exception when calling AuthApi->login: %s\n" % e)
```



### Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **organization_id** | **str**| ID of organization to authenticate into | [optional] 
 **redirect_uri** | **str**| URL to redirect to after successful login | [optional] 
 **inv_code** | **str**| Code sent via email as a result of successful call to send_invitation | [optional] 

### Return type

void (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**303** | Response that redirects to OAuth provider through a Location header to be handled by browser. |  -  |
**400** | OAuth error likely with OIDC provider. |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **logout**
> logout()

Logout

Invalidate your current auth credential stored typically stored in a cookie. This does not invalidate your API key.

### Example


```python
import trieve_py_client
from trieve_py_client.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to https://api.trieve.ai
# See configuration.py for a list of all supported configuration parameters.
configuration = trieve_py_client.Configuration(
    host = "https://api.trieve.ai"
)


# Enter a context with an instance of the API client
with trieve_py_client.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = trieve_py_client.AuthApi(api_client)

    try:
        # Logout
        api_instance.logout()
    except Exception as e:
        print("Exception when calling AuthApi->logout: %s\n" % e)
```



### Parameters

This endpoint does not need any parameter.

### Return type

void (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: Not defined

### HTTP response details

| Status code | Description | Response headers |
|-------------|-------------|------------------|
**204** | Confirmation that your current auth token has been invalidated. This does not invalidate your API key. |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

