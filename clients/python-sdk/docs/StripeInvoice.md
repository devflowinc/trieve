# StripeInvoice


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **datetime** |  | 
**hosted_invoice_url** | **str** |  | 
**id** | **str** |  | 
**org_id** | **str** |  | 
**status** | **str** |  | 
**stripe_id** | **str** |  | [optional] 
**total** | **int** |  | 

## Example

```python
from trieve_py_client.models.stripe_invoice import StripeInvoice

# TODO update the JSON string below
json = "{}"
# create an instance of StripeInvoice from a JSON string
stripe_invoice_instance = StripeInvoice.from_json(json)
# print the JSON string representation of the object
print(StripeInvoice.to_json())

# convert the object into a dict
stripe_invoice_dict = stripe_invoice_instance.to_dict()
# create an instance of StripeInvoice from a dict
stripe_invoice_form_dict = stripe_invoice.from_dict(stripe_invoice_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


