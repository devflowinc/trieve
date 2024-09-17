# StripePlan


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**amount** | **int** |  | 
**chunk_count** | **int** |  | 
**created_at** | **datetime** |  | 
**dataset_count** | **int** |  | 
**file_storage** | **int** |  | 
**id** | **str** |  | 
**message_count** | **int** |  | 
**name** | **str** |  | 
**stripe_id** | **str** |  | 
**updated_at** | **datetime** |  | 
**user_count** | **int** |  | 

## Example

```python
from trieve_py_client.models.stripe_plan import StripePlan

# TODO update the JSON string below
json = "{}"
# create an instance of StripePlan from a JSON string
stripe_plan_instance = StripePlan.from_json(json)
# print the JSON string representation of the object
print(StripePlan.to_json())

# convert the object into a dict
stripe_plan_dict = stripe_plan_instance.to_dict()
# create an instance of StripePlan from a dict
stripe_plan_form_dict = stripe_plan.from_dict(stripe_plan_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


