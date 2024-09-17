# QueryDetails


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**search_id** | **str** |  | 
**type** | **str** |  | 

## Example

```python
from trieve_py_client.models.query_details import QueryDetails

# TODO update the JSON string below
json = "{}"
# create an instance of QueryDetails from a JSON string
query_details_instance = QueryDetails.from_json(json)
# print the JSON string representation of the object
print(QueryDetails.to_json())

# convert the object into a dict
query_details_dict = query_details_instance.to_dict()
# create an instance of QueryDetails from a dict
query_details_form_dict = query_details.from_dict(query_details_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


