# Market

The market estimates the probability of future events by aggregating the
activity of players trading conditional IOUs.

## IOUs

An IOU is a debt for a specified amount that the issuer owes the holder. The
holder of an IOU can unilaterally transfer it to someone else, either in its
entirety or by splitting it into several smaller IOUs that sum to the same
amount as the original. Transferring an IOU back to the issuer is equivalent
to voiding or cancelling it as players cannot hold debt issued by themselves.

IOUs have no maturity date and do not require interest payments. IOUs are
denominated in US dollars, but the issuer has no legal obligation to actually
redeem them for US dollars, or indeed for anything else, although they are
free to do so if they wish.

Players can issue an arbitrary number of IOUs of arbitrary amounts, however
other players may consider the IOUs to be less valuable if they believe that
the issuer has issued too many, or is likely to do so in the future. All IOUs
are public, so players cannot have hidden debts.

## Conditions

IOUs can be conditional, in which case they are considered void if the
condition does not hold. This allows players to make predictions about future
events by trading IOUs which are conditional on those events taking place.

For example, a player might create an IOU with the condition "Donald Trump
will win the 2020 election" and trade it with another player for an IOU that
is contingent on this condition not holding, if they believe that to be more
likely. These IOUs could specify different amounts, to reflect the different
probabilities that each player has estimated for this event. Once the election
takes place and the outcome is revealed, one of the IOUs can be treated as
void and the other can be treated as unconditional.

Some conditions are temporal, in which case the IOU is void if the condition
does not hold by a specified time.

For example, a player might create an IOU with the condition "Atmospheric CO2
levels pass 450ppm" and specify a date of 1 Jan 2030 if they believe that this
is unlikely to be the case.

