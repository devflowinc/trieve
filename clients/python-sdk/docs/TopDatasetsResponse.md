# TopDatasetsResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**dataset_id** | **str** |  | 
**dataset_tracking_id** | **str** |  | [optional] 
**total_queries** | **int** |  | 

## Example

```python
from trieve_py_client.models.top_datasets_response import TopDatasetsResponse

# TODO update the JSON string below
json = "{}"
# create an instance of TopDatasetsResponse from a JSON string
top_datasets_response_instance = TopDatasetsResponse.from_json(json)
# print the JSON string representation of the object
print(TopDatasetsResponse.to_json())

# convert the object into a dict
top_datasets_response_dict = top_datasets_response_instance.to_dict()
# create an instance of TopDatasetsResponse from a dict
top_datasets_response_form_dict = top_datasets_response.from_dict(top_datasets_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


