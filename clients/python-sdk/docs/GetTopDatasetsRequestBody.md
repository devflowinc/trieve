# GetTopDatasetsRequestBody


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**date_range** | [**DateRange**](DateRange.md) |  | [optional] 
**type** | [**TopDatasetsRequestTypes**](TopDatasetsRequestTypes.md) |  | 

## Example

```python
from trieve_py_client.models.get_top_datasets_request_body import GetTopDatasetsRequestBody

# TODO update the JSON string below
json = "{}"
# create an instance of GetTopDatasetsRequestBody from a JSON string
get_top_datasets_request_body_instance = GetTopDatasetsRequestBody.from_json(json)
# print the JSON string representation of the object
print(GetTopDatasetsRequestBody.to_json())

# convert the object into a dict
get_top_datasets_request_body_dict = get_top_datasets_request_body_instance.to_dict()
# create an instance of GetTopDatasetsRequestBody from a dict
get_top_datasets_request_body_form_dict = get_top_datasets_request_body.from_dict(get_top_datasets_request_body_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


