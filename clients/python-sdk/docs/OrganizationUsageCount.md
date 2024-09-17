# OrganizationUsageCount


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunk_count** | **int** |  | 
**dataset_count** | **int** |  | 
**file_storage** | **int** |  | 
**id** | **str** |  | 
**message_count** | **int** |  | 
**org_id** | **str** |  | 
**user_count** | **int** |  | 

## Example

```python
from trieve_py_client.models.organization_usage_count import OrganizationUsageCount

# TODO update the JSON string below
json = "{}"
# create an instance of OrganizationUsageCount from a JSON string
organization_usage_count_instance = OrganizationUsageCount.from_json(json)
# print the JSON string representation of the object
print(OrganizationUsageCount.to_json())

# convert the object into a dict
organization_usage_count_dict = organization_usage_count_instance.to_dict()
# create an instance of OrganizationUsageCount from a dict
organization_usage_count_form_dict = organization_usage_count.from_dict(organization_usage_count_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


