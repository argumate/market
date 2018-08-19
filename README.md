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

## Offers

In order to effect a trade, players must first post an _offer_, which states
the buy and sell prices that a player is willing to accept for a $1 IOU with a
specified condition. The implicit default offer is to buy any IOU for zero
dollars (a bargain!) and sell any IOU for one dollar, as trading a conditional
$1 IOU for one unconditional dollar is always an improvement.

A more realistic example would be to offer to buy a conditional $1 IOU for 50¢
and sell it for 60¢, representing a belief that the probability of the
condition being true is somewhere between 50% and 60%. If another player posts
an offer to sell at ≤50¢ or to buy at ≥60¢ (because they believe the condition
has a different probability of being true) then a trade can take place.

Here is a concrete example of compatible offers leading to a trade:

Offer from Player 1
 - BUY @ 50¢
 - SELL @ 60¢

Offer from Player 2
 - BUY @ 30¢
 - SELL @ 40¢

These offers result in a market clearing fair price of 45¢ for Player 1 to buy
from Player 2. However, making the actual trade involves an exchange of IOUs
with inverse conditions and prices:

 - Player 1 issues a negated conditional IOU for 45¢ to Player 2
 - Player 2 issues a conditional IOU for 55¢ ($1 - 45¢) to Player 1

If the condition is true, then Player 1 will have earned 55¢, equivalent to
purchasing a $1 asset for 45¢. If the condition is false, then Player 2 will
have earned 45¢, equivalent to selling an asset worth nothing for 45¢.

(Note that specifying a very low sell price or very high buy price risks a
large loss for a small gain, so exercise caution!)

This trade can be repeated arbitrarily, depending on each player's appetite
for risk on this particular condition.

If multiple players place offers with the same condition, they will be ranked
by price, so the lowest sell price will match with the highest buy price (with
the actual price for the trade being the midpoint of these two prices). If two
offers have the same price, the offer made earliest will be considered first.

