# GetCrawlOptionsResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**crawl_options** | [**CrawlOptions**](CrawlOptions.md) |  | [optional] 

## Example

```python
from trieve_py_client.models.get_crawl_options_response import GetCrawlOptionsResponse

# TODO update the JSON string below
json = "{}"
# create an instance of GetCrawlOptionsResponse from a JSON string
get_crawl_options_response_instance = GetCrawlOptionsResponse.from_json(json)
# print the JSON string representation of the object
print(GetCrawlOptionsResponse.to_json())

# convert the object into a dict
get_crawl_options_response_dict = get_crawl_options_response_instance.to_dict()
# create an instance of GetCrawlOptionsResponse from a dict
get_crawl_options_response_form_dict = get_crawl_options_response.from_dict(get_crawl_options_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


