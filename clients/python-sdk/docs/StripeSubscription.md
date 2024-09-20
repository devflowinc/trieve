# StripeSubscription


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **datetime** |  | 
**current_period_end** | **datetime** |  | [optional] 
**id** | **str** |  | 
**organization_id** | **str** |  | 
**plan_id** | **str** |  | 
**stripe_id** | **str** |  | 
**updated_at** | **datetime** |  | 

## Example

```python
from trieve_py_client.models.stripe_subscription import StripeSubscription

# TODO update the JSON string below
json = "{}"
# create an instance of StripeSubscription from a JSON string
stripe_subscription_instance = StripeSubscription.from_json(json)
# print the JSON string representation of the object
print(StripeSubscription.to_json())

# convert the object into a dict
stripe_subscription_dict = stripe_subscription_instance.to_dict()
# create an instance of StripeSubscription from a dict
stripe_subscription_form_dict = stripe_subscription.from_dict(stripe_subscription_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


