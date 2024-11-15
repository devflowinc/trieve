# AddToCart


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**event_name** | **str** | The name of the event | 
**event_type** | **str** |  | 
**is_conversion** | **bool** | Whether the event is a conversion event | [optional] 
**items** | **List[str]** | The items that were added to the cart | 
**metadata** | **object** | Any other metadata associated with the event | [optional] 
**request** | [**RequestInfo**](RequestInfo.md) |  | [optional] 
**user_id** | **str** | The user id of the user who added the items to the cart | [optional] 

## Example

```python
from trieve_py_client.models.add_to_cart import AddToCart

# TODO update the JSON string below
json = "{}"
# create an instance of AddToCart from a JSON string
add_to_cart_instance = AddToCart.from_json(json)
# print the JSON string representation of the object
print(AddToCart.to_json())

# convert the object into a dict
add_to_cart_dict = add_to_cart_instance.to_dict()
# create an instance of AddToCart from a dict
add_to_cart_form_dict = add_to_cart.from_dict(add_to_cart_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


