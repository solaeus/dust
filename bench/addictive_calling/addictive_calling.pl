sub increment { $_[0] + 1 }

my $i = 0;
while ($i < 1000000) {
    $i = increment($i);
}
