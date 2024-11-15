# Purchase


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**currency** | **str** | The currency of the purchase | [optional] 
**event_name** | **str** | The name of the event | 
**event_type** | **str** |  | 
**is_conversion** | **bool** | Whether the event is a conversion event | [optional] 
**items** | **List[str]** | The items that were purchased | 
**request** | [**RequestInfo**](RequestInfo.md) |  | [optional] 
**user_id** | **str** | The user id of the user who purchased the items | [optional] 
**value** | **float** | The value of the purchase | [optional] 

## Example

```python
from trieve_py_client.models.purchase import Purchase

# TODO update the JSON string below
json = "{}"
# create an instance of Purchase from a JSON string
purchase_instance = Purchase.from_json(json)
# print the JSON string representation of the object
print(Purchase.to_json())

# convert the object into a dict
purchase_dict = purchase_instance.to_dict()
# create an instance of Purchase from a dict
purchase_form_dict = purchase.from_dict(purchase_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


