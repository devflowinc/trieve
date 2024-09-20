# OrganizationWithSubAndPlan


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**organization** | [**Organization**](Organization.md) |  | 
**plan** | [**StripePlan**](StripePlan.md) |  | [optional] 
**subscription** | [**StripeSubscription**](StripeSubscription.md) |  | [optional] 

## Example

```python
from trieve_py_client.models.organization_with_sub_and_plan import OrganizationWithSubAndPlan

# TODO update the JSON string below
json = "{}"
# create an instance of OrganizationWithSubAndPlan from a JSON string
organization_with_sub_and_plan_instance = OrganizationWithSubAndPlan.from_json(json)
# print the JSON string representation of the object
print(OrganizationWithSubAndPlan.to_json())

# convert the object into a dict
organization_with_sub_and_plan_dict = organization_with_sub_and_plan_instance.to_dict()
# create an instance of OrganizationWithSubAndPlan from a dict
organization_with_sub_and_plan_form_dict = organization_with_sub_and_plan.from_dict(organization_with_sub_and_plan_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


